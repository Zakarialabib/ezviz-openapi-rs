//! Windows FFI bindings for the proprietary **PCOpenSDK** (`OpenNetStream.dll`).
//!
//! This module is compiled only with `--features pc-sdk` on `target_os =
//! "windows"`. It loads the SDK **at runtime** via [`libloading`], so the crate
//! itself stays publishable and never links or redistributes the proprietary
//! DLL — you supply the SDK (e.g. `EZPCOpenSDK_v5.1.18`) on the target machine.
//!
//! The SDK gives the desktop app direct P2P preview and PTZ control without the
//! HTTP round-trip, which is what a signage player wants for low-latency CCTV.
//!
//! ## Loading the SDK
//!
//! `PcSdk::load()` searches, in order:
//! 1. `$EZVIZ_SDK_DIR/lib/win64/OpenNetStream.dll` (and `win32`),
//! 2. `$EZVIZ_SDK_DIR/OpenNetStream.dll`,
//! 3. the process working directory,
//! 4. the system `PATH`.
//!
//! ## Typical flow
//!
//! ```no_run
//! use ezviz_openapi::pc_sdk::{PcSdk, PtzCommand, PtzAction};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let sdk = PcSdk::load()?;
//! sdk.init("YOUR_APP_KEY")?;
//! sdk.set_access_token("at.xxxx")?;
//!
//! let session = sdk.alloc_session_ex(None, std::ptr::null_mut())?;
//! // Start panning up; the SDK is async, watch the message callback for
//! // INS_PTZCTRL_SUCCESS (46) / INS_PTZCTRL_FAILED (47).
//! sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Start, 1)?;
//! sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Stop, 1)?;
//! sdk.free_session(&session)?;
//! sdk.fini_lib()?;
//! # Ok(()) }
//! ```
//!
//! All functions are `unsafe`-free wrappers that return `Result<()>`; the SDK's
//! `0 = success, -1 = failure` convention is mapped to [`crate::EzvizError::Sdk`].

use crate::error::{EzvizError, Result};
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

/// `OPENSDK_RESULT` from `OpenNetStreamDefine.h` — plain `int`.
type OpenSdkResult = c_int;

/// `CALLBACK` on Windows is `__stdcall` → `extern "system"`.
type MessageHandler = extern "system" fn(
    session: *const c_char,
    msg_type: u32,
    error_code: u32,
    message: *const c_char,
    user: *mut c_void,
);

/// PTZ command codes (`enum PTZCommand`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PtzCommand {
    /// Up (`0`).
    Up = 0,
    /// Down (`1`).
    Down = 1,
    /// Left (`2`).
    Left = 2,
    /// Right (`3`).
    Right = 3,
    /// Upper-left (`4`).
    UpperLeft = 4,
    /// Lower-left (`5`).
    LowerLeft = 5,
    /// Upper-right (`6`).
    UpperRight = 6,
    /// Lower-right (`7`).
    LowerRight = 7,
    /// Zoom in (`8`).
    ZoomIn = 8,
    /// Zoom out (`9`).
    ZoomOut = 9,
    /// Focus near (`10`).
    FocusNear = 10,
    /// Focus far (`11`).
    FocusFar = 11,
    /// Iris open / startup (`12`).
    IrisStartup = 12,
    /// Iris close / stop-down (`13`).
    IrisStopDown = 13,
    /// Light on/off (`14`).
    Light = 14,
    /// Wiper on/off (`15`).
    Wiper = 15,
    /// Auto scan (`16`).
    Auto = 16,
}

/// PTZ action (`enum PTZAction`): press = start, release = stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PtzAction {
    /// Press to start the action (`0`).
    Start = 0,
    /// Release to stop the action (`1`).
    Stop = 1,
}

/// Global config keys (`enum ConfigKey`).
pub mod config_key {
    /// UTF-8 data encoding flag (`1`).
    pub const DATA_UTF8: i32 = 1;
    /// Stream transmission flag (`2`).
    pub const OPEN_STREAMTRANS: i32 = 2;
    /// Disable P2P (`3`).
    pub const CLOSE_P2P: i32 = 3;
    /// SDK log level (`4`).
    pub const LOG_LEVEL: i32 = 4;
    /// Maximum P2P connection count (`6`).
    pub const P2P_MAXNUM: i32 = 6;
    /// Disable reporting (`9`).
    pub const CLOSE_REPORT: i32 = 9;
    /// Talk channel setting (`10`).
    pub const TALK_CHANNEL: i32 = 10;
    /// Fast-stream flag (`13`).
    pub const FAST_STREAM: i32 = 13;
    /// Disable `blong` serial checking (`14`).
    pub const CLOSE_BLONG_SERIAL: i32 = 14;
    /// Disable key authentication (`15`).
    pub const CLOSE_KEY_AUTH: i32 = 15;
    /// Slave-data setting (`16`).
    pub const SLAVE_DATA: i32 = 16;
    /// P2P limit (`17`).
    pub const P2P_LIMIT: i32 = 17;
}

/// Message types (`enum MessageType`) relevant to PTZ / playback.
pub mod message_type {
    /// PTZ exception message (`19`).
    pub const PTZ_EXCEPTION: u32 = 19;
    /// PTZ control succeeded message (`46`).
    pub const PTZCTRL_SUCCESS: u32 = 46;
    /// PTZ control failed message (`47`).
    pub const PTZCTRL_FAILED: u32 = 47;
}

/// A loaded `OpenNetStream.dll` handle. `Send + Sync` (the SDK is thread-safe
/// for independent sessions).
pub struct PcSdk {
    lib: Library,
}

impl PcSdk {
    /// Load the SDK from the standard search locations (see module docs).
    pub fn load() -> Result<Self> {
        if !cfg!(target_os = "windows") {
            return Err(EzvizError::Sdk(
                "PCOpenSDK (OpenNetStream.dll) is only supported on Windows".into(),
            ));
        }

        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(dir) = std::env::var("EZVIZ_SDK_DIR") {
            let base = std::path::PathBuf::from(dir);
            candidates.push(base.join("lib").join("win64").join("OpenNetStream.dll"));
            candidates.push(base.join("lib").join("win32").join("OpenNetStream.dll"));
            candidates.push(base.join("OpenNetStream.dll"));
        }
        candidates.push(std::path::PathBuf::from("OpenNetStream.dll"));

        let mut last_err: Option<libloading::Error> = None;
        for path in candidates {
            match unsafe { Library::new(&path) } {
                Ok(lib) => return Ok(PcSdk { lib }),
                Err(e) => last_err = Some(e),
            }
        }
        Err(EzvizError::Sdk(format!(
            "could not load OpenNetStream.dll (set EZVIZ_SDK_DIR): {}",
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "no candidates".into())
        )))
    }

    /// Load from an explicit DLL path.
    pub fn load_from(path: impl AsRef<std::path::Path>) -> Result<Self> {
        if !cfg!(target_os = "windows") {
            return Err(EzvizError::Sdk(
                "PCOpenSDK (OpenNetStream.dll) is only supported on Windows".into(),
            ));
        }

        let lib = unsafe { Library::new(path.as_ref()) }
            .map_err(|e| EzvizError::Sdk(format!("load {}: {e}", path.as_ref().display())))?;
        Ok(PcSdk { lib })
    }

    fn sym<'a, T>(&'a self, name: &str) -> Result<Symbol<'a, T>> {
        unsafe { self.lib.get(name.as_bytes()) }
            .map_err(|e| EzvizError::Sdk(format!("symbol {name}: {e}")))
    }

    fn cstr(s: &str) -> Result<CString> {
        CString::new(s).map_err(|_| EzvizError::Sdk("string contains NUL byte".into()))
    }

    /// `OpenSDK_InitLib(authAddr, platform, appId)` — init with explicit
    /// auth/platform addresses (defaults: `https://openauth.ys7.com`,
    /// `https://open.ys7.com`).
    pub fn init_lib(
        &self,
        auth_addr: &str,
        platform: &str,
        app_id: &str,
    ) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char, *const c_char, *const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_InitLib")?;
        let a = Self::cstr(auth_addr)?;
        let p = Self::cstr(platform)?;
        let id = Self::cstr(app_id)?;
        let rc = unsafe { f(a.as_ptr(), p.as_ptr(), id.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!("OpenSDK_InitLib failed ({rc})")));
        }
        Ok(())
    }

    /// `OpenSDK_Init(appId)` — init with the default (CN) platform.
    pub fn init(&self, app_id: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_Init")?;
        let id = Self::cstr(app_id)?;
        let rc = unsafe { f(id.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!("OpenSDK_Init failed ({rc})")));
        }
        Ok(())
    }

    /// `OpenSDK_FiniLib()`.
    pub fn fini_lib(&self) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn() -> OpenSdkResult> = self.sym("OpenSDK_FiniLib")?;
        let rc = unsafe { f() };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!("OpenSDK_FiniLib failed ({rc})")));
        }
        Ok(())
    }

    /// `OpenSDK_SetAccessToken(token)`.
    pub fn set_access_token(&self, token: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_SetAccessToken")?;
        let t = Self::cstr(token)?;
        let rc = unsafe { f(t.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_SetAccessToken failed ({rc})"
            )));
        }
        Ok(())
    }

    /// `OpenSDK_SetAppID(appId)`.
    pub fn set_app_id(&self, app_id: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_SetAppID")?;
        let id = Self::cstr(app_id)?;
        let rc = unsafe { f(id.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!("OpenSDK_SetAppID failed ({rc})")));
        }
        Ok(())
    }

    /// `OpenSDK_SetClientVer(version)`.
    pub fn set_client_ver(&self, version: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_SetClientVer")?;
        let v = Self::cstr(version)?;
        let rc = unsafe { f(v.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_SetClientVer failed ({rc})"
            )));
        }
        Ok(())
    }

    /// `OpenSDK_SetConfigInfo(key, value)`.
    pub fn set_config_info(&self, key: i32, value: i32) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(i32, i32)> = self.sym("OpenSDK_SetConfigInfo")?;
        unsafe { f(key, value) };
        Ok(())
    }

    /// `OpenSDK_SetMessageCallback(handler)` — install the async message
    /// callback (PTZ results, playback events, …). The handler must be a
    /// `'static` `extern "system"` fn.
    pub fn set_message_callback(&self, handler: Option<MessageHandler>) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(Option<MessageHandler>)> =
            self.sym("OpenSDK_SetMessageCallback")?;
        unsafe { f(handler) };
        Ok(())
    }

    /// `OpenSDK_AllocSessionEx(handler, user, &session, &len)` — allocate a
    /// session id. Returns the session string (SDK-owned memory, do not free).
    pub fn alloc_session_ex(
        &self,
        handler: Option<MessageHandler>,
        user: *mut c_void,
    ) -> Result<String> {
        let f: Symbol<
            unsafe extern "system" fn(
                Option<MessageHandler>,
                *mut c_void,
                *mut *mut c_char,
                *mut c_int,
            ) -> OpenSdkResult,
        > = self.sym("OpenSDK_AllocSessionEx")?;

        let mut buf = [0u8; 128];
        let mut ptr: *mut c_char = buf.as_mut_ptr() as *mut c_char;
        let mut len: c_int = buf.len() as c_int;

        let rc = unsafe { f(handler, user, &mut ptr, &mut len) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_AllocSessionEx failed ({rc})"
            )));
        }
        let session = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        Ok(session)
    }

    /// `OpenSDK_FreeSession(session)`.
    pub fn free_session(&self, session: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_FreeSession")?;
        let s = Self::cstr(session)?;
        let rc = unsafe { f(s.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_FreeSession failed ({rc})"
            )));
        }
        Ok(())
    }

    /// `OpenSDK_SetSessionConfig(session, key, value)`.
    pub fn set_session_config(&self, session: &str, key: i32, value: i32) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char, i32, i32)> =
            self.sym("OpenSDK_SetSessionConfig")?;
        let s = Self::cstr(session)?;
        unsafe { f(s.as_ptr(), key, value) };
        Ok(())
    }

    /// `OpenSDK_StartRealPlayEx(session, hwnd, serial, channel, safeKey)`.
    ///
    /// `hwnd` is the native window handle to render into; pass `0` to only
    /// pull the stream (e.g. to feed a custom renderer via the data callback).
    /// `safe_key` is the video-encryption key, or `None` for unencrypted.
    pub fn start_real_play_ex(
        &self,
        session: &str,
        hwnd: usize,
        device_serial: &str,
        channel_no: i32,
        safe_key: Option<&str>,
    ) -> Result<()> {
        let f: Symbol<
            unsafe extern "system" fn(
                *const c_char,
                usize,
                *const c_char,
                i32,
                *const c_char,
            ) -> OpenSdkResult,
        > = self.sym("OpenSDK_StartRealPlayEx")?;
        let s = Self::cstr(session)?;
        let serial = Self::cstr(device_serial)?;
        let key = safe_key.map(Self::cstr).transpose()?;
        let rc = unsafe {
            f(
                s.as_ptr(),
                hwnd,
                serial.as_ptr(),
                channel_no,
                key.as_ref().map_or(std::ptr::null(), |k| k.as_ptr()),
            )
        };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_StartRealPlayEx failed ({rc})"
            )));
        }
        Ok(())
    }

    /// `OpenSDK_StopRealPlayEx(session)`.
    pub fn stop_real_play_ex(&self, session: &str) -> Result<()> {
        let f: Symbol<unsafe extern "system" fn(*const c_char) -> OpenSdkResult> =
            self.sym("OpenSDK_StopRealPlayEx")?;
        let s = Self::cstr(session)?;
        let rc = unsafe { f(s.as_ptr()) };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_StopRealPlayEx failed ({rc})"
            )));
        }
        Ok(())
    }

    /// `OpenSDK_PTZCtrlEx(session, serial, channel, command, action, speed)`.
    ///
    /// Async: the result arrives on the message callback as
    /// `INS_PTZCTRL_SUCCESS` (46) or `INS_PTZCTRL_FAILED` (47).
    pub fn ptz_ctrl_ex(
        &self,
        session: &str,
        device_serial: &str,
        channel_no: i32,
        command: PtzCommand,
        action: PtzAction,
        speed: i32,
    ) -> Result<()> {
        let f: Symbol<
            unsafe extern "system" fn(
                *const c_char,
                *const c_char,
                i32,
                i32,
                i32,
                i32,
            ) -> OpenSdkResult,
        > = self.sym("OpenSDK_PTZCtrlEx")?;
        let s = Self::cstr(session)?;
        let serial = Self::cstr(device_serial)?;
        let rc = unsafe {
            f(
                s.as_ptr(),
                serial.as_ptr(),
                channel_no,
                command as i32,
                action as i32,
                speed,
            )
        };
        if rc != 0 {
            return Err(EzvizError::Sdk(format!(
                "OpenSDK_PTZCtrlEx failed ({rc})"
            )));
        }
        Ok(())
    }
}

// The SDK is safe to share across threads for independent sessions.
unsafe impl Send for PcSdk {}
unsafe impl Sync for PcSdk {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_pc_sdk_load_non_windows_graceful_error() {
        match PcSdk::load() {
            Err(EzvizError::Sdk(msg)) => {
                assert!(msg.contains("only supported on Windows"));
            }
            _ => panic!("expected EzvizError::Sdk on non-Windows"),
        }
    }
}
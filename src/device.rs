//! Device query APIs (`POST /api/lapp/device/{list,info,status}`,
//! `POST /api/lapp/camera/list`, `POST /api/lapp/device/camera/list`).

use crate::client::EzvizClient;
use crate::error::Result;
use crate::types::ListResponse;
use serde::Deserialize;

/// A device in the current account (`/api/lapp/device/list`).
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceInfo {
    /// Device serial number (`deviceSerial`).
    #[serde(rename = "deviceSerial")]
    pub device_serial: String,
    /// Device name (`deviceName`).
    #[serde(rename = "deviceName")]
    pub device_name: String,
    /// Device type code (`deviceType`).
    #[serde(rename = "deviceType", default)]
    pub device_type: Option<String>,
    /// Online status: 0 = offline, 1 = online.
    #[serde(default)]
    pub status: i32,
    /// Arming/disarming state (A1 devices: 0 Sleep / 8 Home / 16 Away;
    /// others: 0 disarm / 1 arm).
    #[serde(default)]
    pub defence: i32,
    /// Firmware/version string (`deviceVersion`).
    #[serde(rename = "deviceVersion", default)]
    pub device_version: Option<String>,
    /// Whether the device is encrypted (`isEncrypt`).
    #[serde(rename = "isEncrypt", default)]
    pub is_encrypt: Option<i32>,
    /// Last status update time (`updateTime`).
    #[serde(rename = "updateTime", default)]
    pub update_time: Option<String>,
    /// Device model (`model`).
    #[serde(rename = "model", default)]
    pub model: Option<String>,
}

/// Single-device info (`/api/lapp/device/info`).
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceDetail {
    /// Device serial number (`deviceSerial`).
    #[serde(rename = "deviceSerial")]
    pub device_serial: String,
    /// Device name (`deviceName`).
    #[serde(rename = "deviceName")]
    pub device_name: String,
    /// Device model (`model`).
    #[serde(default)]
    pub model: Option<String>,
    /// Online status: 0 = offline, 1 = online.
    #[serde(default)]
    pub status: i32,
    /// Arming/disarming state (0 disarm / 1 arm on most devices).
    #[serde(default)]
    pub defence: i32,
    /// Whether the device is encrypted (`isEncrypt`).
    #[serde(rename = "isEncrypt", default)]
    pub is_encrypt: Option<i32>,
    /// Extra fields the platform may add (channels, capability, …) are kept.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Camera / channel under a device (`/api/lapp/device/camera/list`).
#[derive(Debug, Clone, Deserialize)]
pub struct CameraChannel {
    /// Serial of the device the channel belongs to (`deviceSerial`).
    #[serde(rename = "deviceSerial")]
    pub device_serial: String,
    /// Serial of the IPC bound to this channel (`ipcSerial`).
    #[serde(rename = "ipcSerial", default)]
    pub ipc_serial: Option<String>,
    /// Channel number (`channelNo`).
    #[serde(rename = "channelNo")]
    pub channel_no: i32,
    /// Device name (`deviceName`).
    #[serde(rename = "deviceName", default)]
    pub device_name: Option<String>,
    /// Channel name (`channelName`).
    #[serde(rename = "channelName", default)]
    pub channel_name: Option<String>,
    /// 0 offline, 1 online, -1 not reported.
    #[serde(default)]
    pub status: i32,
    /// Whether the channel is shared (`isShared`).
    #[serde(rename = "isShared", default)]
    pub is_shared: Option<String>,
    /// Channel snapshot image URL (`picUrl`).
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    /// Whether the channel is encrypted (`isEncrypt`).
    #[serde(rename = "isEncrypt", default)]
    pub is_encrypt: Option<i32>,
    /// 0 smooth, 1 BD, 2 HD, 3 FHD.
    #[serde(rename = "videoLevel", default)]
    pub video_level: Option<i32>,
    /// Whether the channel is linked to a related IPC (`relatedIpc`).
    #[serde(rename = "relatedIpc", default)]
    pub related_ipc: Option<bool>,
}

impl EzvizClient {
    /// `POST /api/lapp/device/list` — devices of the current account.
    /// `page_start` begins at 0; `page_size` defaults to 10, max 50.
    pub async fn list_devices(
        &self,
        page_start: i32,
        page_size: i32,
    ) -> Result<ListResponse<DeviceInfo>> {
        let resp = self
            .post_form::<Vec<DeviceInfo>>(
                "/api/lapp/device/list",
                &[
                    ("pageStart", page_start.to_string()),
                    ("pageSize", page_size.to_string()),
                ],
                true,
            )
            .await?;
        if !resp.is_success() {
            return Err(crate::error::EzvizError::Api {
                code: resp.code.to_string(),
                msg: resp.msg.unwrap_or_default(),
            });
        }
        Ok(resp.into())
    }

    /// `POST /api/lapp/device/info` — single device basic info.
    pub async fn device_info(&self, device_serial: &str) -> Result<DeviceDetail> {
        self.call(
            "/api/lapp/device/info",
            &[("deviceSerial", device_serial.to_string())],
        )
        .await
    }

    /// `POST /api/lapp/device/status` — device online/capability status.
    /// Response fields vary by device family, so known keys are typed and the
    /// remainder is preserved in `extra`.
    pub async fn device_status(&self, device_serial: &str) -> Result<DeviceDetail> {
        self.call(
            "/api/lapp/device/status",
            &[("deviceSerial", device_serial.to_string())],
        )
        .await
    }

    /// `POST /api/lapp/camera/list` — all cameras of the current account.
    pub async fn list_cameras(&self) -> Result<Vec<CameraChannel>> {
        self.call("/api/lapp/camera/list", &[]).await
    }

    /// `POST /api/lapp/device/camera/list` — channels of one device (NVR
    /// channel enumeration included).
    pub async fn device_cameras(&self, device_serial: &str) -> Result<Vec<CameraChannel>> {
        self.call(
            "/api/lapp/device/camera/list",
            &[("deviceSerial", device_serial.to_string())],
        )
        .await
    }
}

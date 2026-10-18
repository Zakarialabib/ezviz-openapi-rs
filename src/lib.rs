//! # ezviz-openapi
//!
//! Async Rust client for the **EZVIZ Open Platform** HTTP interface
//! (`open.ys7.com` / `isgpopen.ezvizlife.com`, a.k.a. `ezvizlife` / `eziot`).
//!
//! Covers the documented `/api/lapp/**` surface:
//!
//! - **Auth** — `POST /api/lapp/token/get` with auto-cache and refresh-on-`10002`.
//! - **Devices** — list, single info, status, camera/channel list.
//! - **Live** — obtain/invalidate HLS/RTMP/FLV/ezopen play addresses (incl. local
//!   and cloud playback addresses via `type`).
//! - **PTZ** — pan/tilt/zoom start & stop, mirror flip, presets (add/move/clear).
//! - **Alarms** — alarm list with motion/IO/event types.
//! - **Capture** — snapshot request.
//!
//! Optional `pc-sdk` feature adds a **Windows FFI binding** for the proprietary
//! PCOpenSDK (`OpenNetStream.dll`), giving the desktop app direct P2P preview
//! and PTZ control without the HTTP hop.
//!
//! ## Quick start
//!
//! ```no_run
//! use ezviz_openapi::{EzvizClient, ptz::{PtzDirection, PtzSpeed}};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = EzvizClient::new("YOUR_APP_KEY", "YOUR_APP_SECRET");
//!
//! // Token is fetched and cached automatically on first use.
//! let devices = client.list_devices(0, 10).await?;
//! for d in devices.items {
//!     println!("{} {} status={}", d.device_serial, d.device_name, d.status);
//! }
//!
//! // Pan the PTZ camera up at medium speed.
//! client.ptz_start("DEVICE_SERIAL", 1, PtzDirection::Up, PtzSpeed::Medium).await?;
//! client.ptz_stop("DEVICE_SERIAL", 1, PtzDirection::Up).await?;
//! # Ok(()) }
//! ```
//!
//! See [`docs/api-reference.md`](https://github.com/your-org/ezviz-openapi-rs/blob/main/docs/api-reference.md)
//! for the full endpoint table, error codes and a flow diagram of the API.

pub mod alarm;
pub mod capture;
pub mod client;
pub mod device;
pub mod error;
pub mod live;
pub mod playback;
pub mod ptz;
pub mod token;
pub mod types;

#[cfg(all(feature = "pc-sdk", target_os = "windows"))]
pub mod pc_sdk;

pub use client::EzvizClient;
pub use error::{EzvizError, Result};
pub use token::AccessTokenInfo;
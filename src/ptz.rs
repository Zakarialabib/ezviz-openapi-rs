//! PTZ control and presets.
//!
//! HTTP surface (`/api/lapp/device/ptz/*`, `/api/lapp/device/preset/*`):
//! start/stop pan-tilt-zoom, mirror flip, and preset add/move/clear.
//!
//! Direction codes follow the platform table:
//! `0 Up, 1 Down, 2 Left, 3 Right, 4 Upper-left, 5 Lower-left,
//!  6 Upper-right, 7 Lower-right, 8 Zoom in, 9 Zoom out,
//!  10 Focus near, 11 Focus far`.

use crate::client::EzvizClient;
use crate::error::Result;
use serde::Deserialize;

/// PTZ direction / command for the HTTP API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtzDirection {
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
}

impl PtzDirection {
    /// Raw `direction` value sent to the API.
    pub fn as_raw(self) -> i32 {
        self as i32
    }
}

/// PTZ speed for the HTTP API: 0 slow, 1 medium, 2 fast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtzSpeed {
    /// Slow (`0`).
    Slow = 0,
    /// Medium (`1`).
    Medium = 1,
    /// Fast (`2`).
    Fast = 2,
}

impl PtzSpeed {
    /// Raw `speed` value sent to the API.
    pub fn as_raw(self) -> i32 {
        self as i32
    }
}

/// Mirror-flip command (`/api/lapp/device/ptz/mirror`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorCommand {
    /// Flip up/down.
    Vertical = 0,
    /// Flip left/right.
    Horizontal = 1,
    /// Center / reset.
    Center = 2,
}

/// Response of `POST /api/lapp/device/preset/add` — the assigned preset index.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetAdd {
    /// Assigned preset index (`index`).
    pub index: i32,
}

impl EzvizClient {
    /// Start a PTZ movement. Call [`ptz_stop`](EzvizClient::ptz_stop) with the
    /// same direction to halt it (the platform requires an explicit stop).
    pub async fn ptz_start(
        &self,
        device_serial: &str,
        channel_no: i32,
        direction: PtzDirection,
        speed: PtzSpeed,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/ptz/start",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("direction", direction.as_raw().to_string()),
                ("speed", speed.as_raw().to_string()),
            ],
        )
        .await
    }

    /// Stop a PTZ movement.
    pub async fn ptz_stop(
        &self,
        device_serial: &str,
        channel_no: i32,
        direction: PtzDirection,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/ptz/stop",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("direction", direction.as_raw().to_string()),
            ],
        )
        .await
    }

    /// Mirror flip (device must support it).
    pub async fn ptz_mirror(
        &self,
        device_serial: &str,
        channel_no: i32,
        command: MirrorCommand,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/ptz/mirror",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("command", (command as i32).to_string()),
            ],
        )
        .await
    }

    /// Add a preset at the current position. Returns the assigned preset index
    /// (1..=12 on C6-family devices) — save it for later recall.
    pub async fn preset_add(&self, device_serial: &str, channel_no: i32) -> Result<i32> {
        let resp: PresetAdd = self
            .call(
                "/api/lapp/device/preset/add",
                &[
                    ("deviceSerial", device_serial.to_string()),
                    ("channelNo", channel_no.to_string()),
                ],
            )
            .await?;
        Ok(resp.index)
    }

    /// Recall (move to) a preset.
    pub async fn preset_move(
        &self,
        device_serial: &str,
        channel_no: i32,
        index: i32,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/preset/move",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("index", index.to_string()),
            ],
        )
        .await
    }

    /// Clear a preset.
    pub async fn preset_clear(
        &self,
        device_serial: &str,
        channel_no: i32,
        index: i32,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/preset/clear",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("index", index.to_string()),
            ],
        )
        .await
    }
}

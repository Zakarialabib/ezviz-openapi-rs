//! Snapshot capture (`POST /api/lapp/device/capture`).
//!
//! Requests the device to capture a picture. The picture is delivered through
//! the platform's picture service (see the alarm/device picture URLs), not
//! returned inline.

use crate::client::EzvizClient;
use crate::error::Result;

impl EzvizClient {
    /// Ask the device to take a snapshot.
    pub async fn capture(&self, device_serial: &str, channel_no: i32) -> Result<()> {
        self.call_ack(
            "/api/lapp/device/capture",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
            ],
        )
        .await
    }
}
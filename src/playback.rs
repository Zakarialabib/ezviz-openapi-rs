//! Playback convenience wrappers.
//!
//! The platform serves local and cloud (CloudPlay) recording playback through
//! the same play-address endpoint (`/api/lapp/live/address/get`) by setting
//! `type` to 2 (local) or 3 (cloud) and passing `startTime`/`stopTime`.

use crate::client::EzvizClient;
use crate::error::Result;
use crate::live::{LiveAddress, LiveType, StreamProtocol, VideoQuality};

impl EzvizClient {
    /// HLS address for a local-recording playback window.
    pub async fn local_playback_address(
        &self,
        device_serial: &str,
        channel_no: i32,
        quality: VideoQuality,
        start_time: &str,
        stop_time: &str,
    ) -> Result<LiveAddress> {
        self.live_address(
            device_serial,
            channel_no,
            StreamProtocol::Hls,
            quality,
            LiveType::LocalPlayback,
            None,
            None,
            Some(start_time),
            Some(stop_time),
        )
        .await
    }

    /// HLS address for a cloud (CloudPlay) recording playback window.
    pub async fn cloud_playback_address(
        &self,
        device_serial: &str,
        channel_no: i32,
        quality: VideoQuality,
        start_time: &str,
        stop_time: &str,
    ) -> Result<LiveAddress> {
        self.live_address(
            device_serial,
            channel_no,
            StreamProtocol::Hls,
            quality,
            LiveType::CloudPlayback,
            None,
            None,
            Some(start_time),
            Some(stop_time),
        )
        .await
    }
}

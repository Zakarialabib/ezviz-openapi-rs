//! Live play-address APIs (`POST /api/lapp/live/address/get` and
//! `POST /api/lapp/live/address/disable`), including local/cloud playback
//! addresses selected through `type` + `startTime`/`stopTime`.

use crate::client::EzvizClient;
use crate::error::Result;
use serde::Deserialize;

/// Stream protocol selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamProtocol {
    Ezopen = 1,
    Hls = 2,
    Rtmp = 3,
    Flv = 4,
}

impl StreamProtocol {
    pub fn as_raw(self) -> i32 {
        self as i32
    }
}

/// Address kind: preview, local recording playback or cloud (CloudPlay) playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveType {
    Preview = 1,
    LocalPlayback = 2,
    CloudPlayback = 3,
}

/// Video quality: main or sub bitrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoQuality {
    Hd = 1,
    Fluent = 2,
}

/// Result of `POST /api/lapp/live/address/get`.
#[derive(Debug, Clone, Deserialize)]
pub struct LiveAddress {
    #[serde(default)]
    pub id: Option<String>,
    pub url: String,
    #[serde(rename = "expireTime", default)]
    pub expire_time: Option<String>,
}

impl EzvizClient {
    /// Obtain a play address. `protocol` selects HLS/RTMP/FLV/ezopen; for
    /// unencrypted ezopen URLs pass `None` for `safe_code`.
    ///
    /// For playback (`LiveType::LocalPlayback`/`CloudPlayback`) pass
    /// `start_time`/`stop_time` as `yyyy-MM-dd HH:mm:ss` strings, and `< 32`
    /// as `expire_time_secs` is interpreted as "let the platform decide".
    #[allow(clippy::too_many_arguments)]
    pub async fn live_address(
        &self,
        device_serial: &str,
        channel_no: i32,
        protocol: StreamProtocol,
        quality: VideoQuality,
        live_type: LiveType,
        expire_time_secs: Option<i64>,
        safe_code: Option<&str>,
        start_time: Option<&str>,
        stop_time: Option<&str>,
    ) -> Result<LiveAddress> {
        let mut params = vec![
            ("deviceSerial", device_serial.to_string()),
            ("channelNo", channel_no.to_string()),
            ("protocol", protocol.as_raw().to_string()),
            ("quality", (quality as i32).to_string()),
            ("type", (live_type as i32).to_string()),
        ];
        if let Some(t) = expire_time_secs {
            params.push(("expireTime", t.to_string()));
        }
        if let Some(code) = safe_code {
            params.push(("code", code.to_string()));
        }
        if let Some(t) = start_time {
            params.push(("startTime", t.to_string()));
        }
        if let Some(t) = stop_time {
            params.push(("stopTime", t.to_string()));
        }
        self.call("/api/lapp/live/address/get", &params).await
    }

    /// Preview HLS address for a camera (the practical default for signage
    /// players: a plain `.m3u8` you can hand to any HLS player).
    pub async fn live_hls_address(
        &self,
        device_serial: &str,
        channel_no: i32,
        quality: VideoQuality,
    ) -> Result<LiveAddress> {
        self.live_address(
            device_serial,
            channel_no,
            StreamProtocol::Hls,
            quality,
            LiveType::Preview,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Invalidate a previously issued play address (`/api/lapp/live/address/disable`).
    pub async fn disable_live_address(
        &self,
        device_serial: &str,
        channel_no: i32,
        url_id: &str,
    ) -> Result<()> {
        self.call_ack(
            "/api/lapp/live/address/disable",
            &[
                ("deviceSerial", device_serial.to_string()),
                ("channelNo", channel_no.to_string()),
                ("urlId", url_id.to_string()),
            ],
        )
        .await
    }
}
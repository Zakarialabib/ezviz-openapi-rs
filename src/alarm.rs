//! Alarm query (`POST /api/lapp/alarm/list`).

use crate::client::EzvizClient;
use crate::error::Result;
use crate::types::ListResponse;
use serde::Deserialize;

/// One alarm record.
#[derive(Debug, Clone, Deserialize)]
pub struct AlarmItem {
    #[serde(rename = "alarmId")]
    pub alarm_id: String,
    #[serde(rename = "alarmName", default)]
    pub alarm_name: Option<String>,
    #[serde(rename = "alarmType")]
    pub alarm_type: i32,
    /// Alarm time, Unix **milliseconds**.
    #[serde(rename = "alarmTime")]
    pub alarm_time: i64,
    #[serde(rename = "channelNo", default)]
    pub channel_no: Option<i32>,
    #[serde(rename = "isEncrypt", default)]
    pub is_encrypt: Option<i32>,
    #[serde(rename = "isChecked", default)]
    pub is_checked: Option<i32>,
    #[serde(rename = "preTime", default)]
    pub pre_time: Option<i32>,
    #[serde(rename = "delayTime", default)]
    pub delay_time: Option<i32>,
    #[serde(rename = "deviceSerial", default)]
    pub device_serial: Option<String>,
    #[serde(rename = "alarmPicUrl", default)]
    pub alarm_pic_url: Option<String>,
    #[serde(rename = "relationAlarms", default)]
    pub relation_alarms: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Alarm-type filter. `-1` = all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmType {
    All = -1,
    PirEvent = 10000,
    MotionDetection = 10002,
    VideoTampering = 10011,
    VideoLoss = 10012,
    LineCrossing = 10013,
    Intrusion = 10014,
    FaceDetection = 10015,
    DoorBellRing = 10016,
    SceneChange = 10020,
    Defocus = 10021,
    AudioException = 10022,
    UnattendedBaggage = 10023,
    ObjectRemoval = 10024,
    IllegalParking = 10025,
    PeopleGathering = 10026,
    Loitering = 10027,
    FastMoving = 10028,
    RegionEntrance = 10029,
    RegionExiting = 10030,
    LowBattery = 10032,
    IoAlarm = 10100,
}

/// Read-state filter: 0 unread, 1 read, 2 all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmStatus {
    Unread = 0,
    Read = 1,
    All = 2,
}

impl EzvizClient {
    /// `POST /api/lapp/alarm/list`. `start_time`/`end_time` are Unix
    /// **milliseconds**; only the last 7 days are searchable.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_alarms(
        &self,
        start_time: Option<i64>,
        end_time: Option<i64>,
        alarm_type: AlarmType,
        status: AlarmStatus,
        page_start: i32,
        page_size: i32,
    ) -> Result<ListResponse<AlarmItem>> {
        let mut params = vec![
            ("alarmType", (alarm_type as i32).to_string()),
            ("status", (status as i32).to_string()),
            ("pageStart", page_start.to_string()),
            ("pageSize", page_size.to_string()),
        ];
        if let Some(t) = start_time {
            params.push(("startTime", t.to_string()));
        }
        if let Some(t) = end_time {
            params.push(("endTime", t.to_string()));
        }
        let resp = self
            .post_form::<Vec<AlarmItem>>("/api/lapp/alarm/list", &params, true)
            .await?;
        if !resp.is_success() {
            return Err(crate::error::EzvizError::Api {
                code: resp.code.to_string(),
                msg: resp.msg.unwrap_or_default(),
            });
        }
        Ok(resp.into())
    }
}
//! Alarm query (`POST /api/lapp/alarm/list`).

use crate::client::EzvizClient;
use crate::error::Result;
use crate::types::ListResponse;
use serde::Deserialize;

/// One alarm record.
#[derive(Debug, Clone, Deserialize)]
pub struct AlarmItem {
    /// Alarm id (`alarmId`).
    #[serde(rename = "alarmId")]
    pub alarm_id: String,
    /// Alarm name (`alarmName`).
    #[serde(rename = "alarmName", default)]
    pub alarm_name: Option<String>,
    /// Alarm type code (`alarmType`); see [`AlarmType`].
    #[serde(rename = "alarmType")]
    pub alarm_type: i32,
    /// Alarm time, Unix **milliseconds**.
    #[serde(rename = "alarmTime")]
    pub alarm_time: i64,
    /// Channel number (`channelNo`).
    #[serde(rename = "channelNo", default)]
    pub channel_no: Option<i32>,
    /// Whether the channel is encrypted (`isEncrypt`).
    #[serde(rename = "isEncrypt", default)]
    pub is_encrypt: Option<i32>,
    /// Read state: `0` unread, `1` read (`isChecked`).
    #[serde(rename = "isChecked", default)]
    pub is_checked: Option<i32>,
    /// `preTime` field reported by the platform.
    #[serde(rename = "preTime", default)]
    pub pre_time: Option<i32>,
    /// `delayTime` field reported by the platform.
    #[serde(rename = "delayTime", default)]
    pub delay_time: Option<i32>,
    /// Serial of the device the alarm belongs to (`deviceSerial`).
    #[serde(rename = "deviceSerial", default)]
    pub device_serial: Option<String>,
    /// URL of the alarm snapshot image (`alarmPicUrl`).
    #[serde(rename = "alarmPicUrl", default)]
    pub alarm_pic_url: Option<String>,
    /// Related alarm records, kept raw (`relationAlarms`).
    #[serde(rename = "relationAlarms", default)]
    pub relation_alarms: Vec<serde_json::Value>,
    /// Extra fields the platform may add are kept.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Alarm-type filter. `-1` = all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmType {
    /// All alarm types (`-1`).
    All = -1,
    /// PIR event (`10000`).
    PirEvent = 10000,
    /// Motion detection (`10002`).
    MotionDetection = 10002,
    /// Video tampering (`10011`).
    VideoTampering = 10011,
    /// Video loss (`10012`).
    VideoLoss = 10012,
    /// Line crossing (`10013`).
    LineCrossing = 10013,
    /// Intrusion detection (`10014`).
    Intrusion = 10014,
    /// Face detection (`10015`).
    FaceDetection = 10015,
    /// Doorbell ring (`10016`).
    DoorBellRing = 10016,
    /// Scene change (`10020`).
    SceneChange = 10020,
    /// Defocus (`10021`).
    Defocus = 10021,
    /// Audio exception (`10022`).
    AudioException = 10022,
    /// Unattended baggage (`10023`).
    UnattendedBaggage = 10023,
    /// Object removal (`10024`).
    ObjectRemoval = 10024,
    /// Illegal parking (`10025`).
    IllegalParking = 10025,
    /// People gathering (`10026`).
    PeopleGathering = 10026,
    /// Loitering (`10027`).
    Loitering = 10027,
    /// Fast moving object (`10028`).
    FastMoving = 10028,
    /// Region entrance (`10029`).
    RegionEntrance = 10029,
    /// Region exiting (`10030`).
    RegionExiting = 10030,
    /// Low battery (`10032`).
    LowBattery = 10032,
    /// I/O alarm (`10100`).
    IoAlarm = 10100,
}

/// Read-state filter: 0 unread, 1 read, 2 all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmStatus {
    /// Unread alarms (`0`).
    Unread = 0,
    /// Read alarms (`1`).
    Read = 1,
    /// Any read state (`2`).
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
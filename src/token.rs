//! Access-token management (`POST /api/lapp/token/get`).

use serde::Deserialize;

/// Response of `POST /api/lapp/token/get`.
///
/// `area_domain` is the region-scoped base URL the token is valid against
/// (e.g. `https://iusopen.ezvizlife.com`). The client auto-switches to it.
#[derive(Debug, Clone, Deserialize)]
pub struct AccessTokenInfo {
    /// Bearer token sent as the `accessToken` form parameter.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Expiry as Unix **milliseconds**.
    #[serde(rename = "expireTime")]
    pub expire_time: i64,
    /// Region-scoped base URL the token is valid against (`areaDomain`).
    #[serde(rename = "areaDomain")]
    pub area_domain: Option<String>,
}
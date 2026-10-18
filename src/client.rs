//! The `EzvizClient`: auth, region switching, form-POST plumbing.

use crate::error::{EzvizError, Result};
use crate::token::AccessTokenInfo;
use crate::types::ApiResponse;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Default international open-platform host.
pub const DEFAULT_BASE_URL: &str = "https://isgpopen.ezvizlife.com";

/// How long an in-memory token is reused before a refresh is attempted
/// (the platform issues tokens valid for 7 days; we refresh daily).
const TOKEN_TTL: Duration = Duration::from_secs(24 * 3600);

struct TokenEntry {
    token: String,
    fetched_at: Instant,
}

/// Async client for the EZVIZ Open Platform HTTP API.
///
/// All methods POST `application/x-www-form-urlencoded` bodies, mirroring the
/// official documentation and SDKs. The client is cheap to clone (it wraps a
/// `reqwest::Client` + shared token cache) and is `Send + Sync`.
#[derive(Clone)]
pub struct EzvizClient {
    app_key: String,
    app_secret: String,
    http: reqwest::Client,
    base_url: std::sync::Arc<RwLock<String>>,
    token: std::sync::Arc<RwLock<Option<TokenEntry>>>,
}

impl EzvizClient {
    /// Create a client that fetches and caches the access token from
    /// `app_key` + `app_secret` on first use (`POST /api/lapp/token/get`).
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_key: app_key.into(),
            app_secret: app_secret.into(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client build cannot fail with a timeout only"),
            base_url: std::sync::Arc::new(RwLock::new(DEFAULT_BASE_URL.to_string())),
            token: std::sync::Arc::new(RwLock::new(None)),
        }
    }

    /// Create a client with a token you already hold (e.g. issued server-side).
    /// The token is used as-is and never refreshed.
    pub fn with_token(access_token: impl Into<String>, area_domain: Option<String>) -> Self {
        let client = Self {
            app_key: String::new(),
            app_secret: String::new(),
            http: reqwest::Client::new(),
            base_url: std::sync::Arc::new(RwLock::new(
                area_domain
                    .clone()
                    .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            )),
            token: std::sync::Arc::new(RwLock::new(None)),
        };
        *client.token.write().expect("token lock") = Some(TokenEntry {
            token: access_token.into(),
            fetched_at: Instant::now(),
        });
        client
    }

    /// Override the region base URL (normally taken from `areaDomain`).
    pub fn set_base_url(&self, url: impl Into<String>) {
        *self.base_url.write().expect("base_url lock") = url.into();
    }

    /// The current region base URL.
    pub fn base_url(&self) -> String {
        self.base_url.read().expect("base_url lock").clone()
    }

    /// The cached access token, fetching one if absent/expired.
    pub async fn access_token(&self) -> Result<String> {
        {
            let guard = self.token.read().expect("token lock");
            if let Some(entry) = guard.as_ref() {
                if entry.fetched_at.elapsed() < TOKEN_TTL {
                    return Ok(entry.token.clone());
                }
            }
        }
        self.refresh_token().await
    }

    /// Force a fresh token via `POST /api/lapp/token/get`.
    pub async fn refresh_token(&self) -> Result<String> {
        let resp: ApiResponse<AccessTokenInfo> = self
            .http
            .post(self.base_url() + "/api/lapp/token/get")
            .form(&[
                ("appKey", self.app_key.as_str()),
                ("appSecret", self.app_secret.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let info = resp.data.ok_or_else(|| EzvizError::Api {
            code: resp.code.to_string(),
            msg: resp
                .msg
                .clone()
                .unwrap_or_else(|| "missing token data".into()),
        })?;

        if let Some(domain) = info.area_domain.clone() {
            self.set_base_url(domain);
        }

        *self.token.write().expect("token lock") = Some(TokenEntry {
            token: info.access_token.clone(),
            fetched_at: Instant::now(),
        });
        Ok(info.access_token)
    }

    /// Low-level form POST. `path` is everything after the base URL (e.g.
    /// `/api/lapp/device/list`). `params` are merged with the access token
    /// unless `requires_token` is false.
    pub async fn post_form<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
        requires_token: bool,
    ) -> Result<ApiResponse<T>> {
        let mut body: Vec<(&str, String)> = params.to_vec();
        if requires_token {
            body.push(("accessToken", self.access_token().await?));
        }

        let resp: ApiResponse<T> = self
            .http
            .post(self.base_url() + path)
            .form(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        // Token expired/mismatched mid-call: refresh once and retry.
        if requires_token && resp.code.as_str() == "10002" {
            self.refresh_token().await?;
            let mut body2 = params.to_vec();
            body2.push(("accessToken", self.access_token().await?));
            return Ok(self
                .http
                .post(self.base_url() + path)
                .form(&body2)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?);
        }

        Ok(resp)
    }

    /// `post_form` + envelope check: returns `Err(EzvizError::Api)` on non-200.
    pub async fn call<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        let resp = self.post_form::<T>(path, params, true).await?;
        if !resp.is_success() {
            return Err(EzvizError::Api {
                code: resp.code.to_string(),
                msg: resp.msg.unwrap_or_else(|| "no message".to_string()),
            });
        }
        resp.data.ok_or_else(|| EzvizError::Api {
            code: resp.code.to_string(),
            msg: "response carried no data field".to_string(),
        })
    }

    /// Like [`call`](EzvizClient::call) but for endpoints whose success reply
    /// has no `data` field (pure acks such as PTZ start/stop).
    pub async fn call_ack(&self, path: &str, params: &[(&str, String)]) -> Result<()> {
        let resp = self
            .post_form::<serde_json::Value>(path, params, true)
            .await?;
        if !resp.is_success() {
            return Err(EzvizError::Api {
                code: resp.code.to_string(),
                msg: resp.msg.unwrap_or_else(|| "no message".to_string()),
            });
        }
        Ok(())
    }
}
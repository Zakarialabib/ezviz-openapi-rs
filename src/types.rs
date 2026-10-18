//! Shared types: response envelope, paging, flexible API codes.

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The EZVIZ API returns its `code` as a *string* (`"200"`) in most docs but
/// some endpoints/regions return a bare integer. This type accepts both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiCode(pub String);

impl ApiCode {
    /// A `200` code means the operation succeeded.
    pub fn is_ok(&self) -> bool {
        self.0 == "200"
    }

    /// The code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ApiCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ApiCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CodeVisitor;
        impl Visitor<'_> for CodeVisitor {
            type Value = ApiCode;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a numeric or string API code")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<ApiCode, E> {
                Ok(ApiCode(v.to_string()))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<ApiCode, E> {
                Ok(ApiCode(v.to_string()))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<ApiCode, E> {
                Ok(ApiCode(v.to_string()))
            }
        }
        deserializer.deserialize_any(CodeVisitor)
    }
}

/// Paging metadata returned at the top level of list endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageInfo {
    /// Total number of records (`total`).
    pub total: i64,
    /// Current page index (`page`).
    #[serde(rename = "page")]
    pub page: i64,
    /// Page size (`size`).
    pub size: i64,
}

/// Standard EZVIZ JSON envelope: `{ "code", "msg", "data"?, "page"? }`.
///
/// `data` is optional — pure-ack endpoints return only `code`/`msg` — and
/// `page` only appears when `T` is a paged list.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    /// Platform status code (`code`); `200` means success.
    pub code: ApiCode,
    /// Status message (`msg`), present on most responses.
    pub msg: Option<String>,
    /// Response payload (`data`), absent on pure-ack endpoints.
    pub data: Option<T>,
    /// Paging metadata (`page`), present on paged list responses.
    pub page: Option<PageInfo>,
}

impl<T> ApiResponse<T> {
    /// True when the platform accepted the operation.
    pub fn is_success(&self) -> bool {
        self.code.is_ok()
    }
}

/// A paged list result: items plus paging info (when reported).
#[derive(Debug, Clone)]
pub struct ListResponse<T> {
    /// Result items.
    pub items: Vec<T>,
    /// Paging metadata, when the platform reports it.
    pub page: Option<PageInfo>,
}

impl<T> From<ApiResponse<Vec<T>>> for ListResponse<T> {
    fn from(resp: ApiResponse<Vec<T>>) -> Self {
        ListResponse {
            items: resp.data.unwrap_or_default(),
            page: resp.page,
        }
    }
}
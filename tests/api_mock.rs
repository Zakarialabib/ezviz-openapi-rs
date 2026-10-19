//! Integration tests against a local mock of the EZVIZ Open Platform.
//! No real credentials or network are used.

use ezviz_openapi::live::{StreamProtocol, VideoQuality};
use ezviz_openapi::ptz::{PtzDirection, PtzSpeed};
use ezviz_openapi::EzvizClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mount the token endpoint returning a fixed token + area domain.
async fn mount_token(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/lapp/token/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "accessToken": "at.testtoken123",
                "expireTime": 9999999999999i64,
                "areaDomain": server.uri()
            },
            "code": "200",
            "msg": "Operating succeeded!"
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn token_is_fetched_and_cached() {
    let server = MockServer::start().await;
    mount_token(&server).await;

    let token_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = token_calls.clone();
    Mock::given(method("POST"))
        .and(path("/api/lapp/device/list"))
        .respond_with(move |req: &wiremock::Request| {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let body = String::from_utf8_lossy(&req.body);
            assert!(body.contains("accessToken=at.testtoken123"), "body: {body}");
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "page": {"total": 1, "page": 0, "size": 10},
                "data": [{
                    "deviceSerial": "427734000",
                    "deviceName": "PTZ Cam",
                    "deviceType": "C6",
                    "status": 1,
                    "defence": 1,
                    "deviceVersion": "V5.3.0 build 150824"
                }],
                "code": "200",
                "msg": "Operating succeeded!"
            }))
        })
        .mount(&server)
        .await;

    let client = EzvizClient::new("appkey", "appsecret");
    client.set_base_url(server.uri());

    let list = client.list_devices(0, 10).await.expect("list_devices");
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].device_serial, "427734000");
    assert_eq!(list.items[0].status, 1);
    assert_eq!(token_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn ptz_start_sends_direction_and_speed() {
    let server = MockServer::start().await;
    mount_token(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/lapp/device/ptz/start"))
        .respond_with(move |req: &wiremock::Request| {
            let body = String::from_utf8_lossy(&req.body);
            assert!(body.contains("direction=2"), "body: {body}");
            assert!(body.contains("speed=1"), "body: {body}");
            assert!(body.contains("deviceSerial=502608888"), "body: {body}");
            assert!(body.contains("channelNo=1"), "body: {body}");
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": "200", "msg": "Operating succeeded!"
            }))
        })
        .mount(&server)
        .await;

    let client = EzvizClient::new("appkey", "appsecret");
    client.set_base_url(server.uri());
    client
        .ptz_start("502608888", 1, PtzDirection::Left, PtzSpeed::Medium)
        .await
        .expect("ptz_start");
}

#[tokio::test]
async fn live_hls_address_parses() {
    let server = MockServer::start().await;
    mount_token(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/lapp/live/address/get"))
        .respond_with(move |req: &wiremock::Request| {
            let body = String::from_utf8_lossy(&req.body);
            assert!(body.contains("protocol=2"), "body: {body}");
            assert!(body.contains("quality=1"), "body: {body}");
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "msg": "Operation succeeded",
                "code": "200",
                "data": {
                    "id": "512628410958159872",
                    "url": "https://iusopen.ezvizlife.com/v3/openlive/F00497273_1_1.m3u8?expire=1668578537",
                    "expireTime": "2022-11-16 06:02:17"
                }
            }))
        })
        .mount(&server)
        .await;

    let client = EzvizClient::new("appkey", "appsecret");
    client.set_base_url(server.uri());
    let addr = client
        .live_address(
            "F00497273",
            1,
            StreamProtocol::Hls,
            VideoQuality::Hd,
            ezviz_openapi::live::LiveType::Preview,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("live_address");
    assert!(addr.url.contains(".m3u8"));
    assert_eq!(addr.id.as_deref(), Some("512628410958159872"));
}

#[tokio::test]
async fn api_error_is_surfaced() {
    let server = MockServer::start().await;
    mount_token(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/lapp/device/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": "20002", "msg": "The device does not exist."
        })))
        .mount(&server)
        .await;

    let client = EzvizClient::new("appkey", "appsecret");
    client.set_base_url(server.uri());
    let err = client.device_info("nope").await.unwrap_err();
    match err {
        ezviz_openapi::EzvizError::Api { code, .. } => assert_eq!(code, "20002"),
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn preset_add_returns_index() {
    let server = MockServer::start().await;
    mount_token(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/lapp/device/preset/add"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"index": 3},
            "code": "200", "msg": "Operating succeeded!"
        })))
        .mount(&server)
        .await;

    let client = EzvizClient::new("appkey", "appsecret");
    client.set_base_url(server.uri());
    let idx = client.preset_add("568261888", 1).await.expect("preset_add");
    assert_eq!(idx, 3);
}

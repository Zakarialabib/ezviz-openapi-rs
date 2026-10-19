//! Print the HLS live URL for a camera (feed it to any HLS player / signage).
//! Run: `EZVIZ_APP_KEY=... EZVIZ_APP_SECRET=... EZVIZ_DEVICE_SERIAL=... cargo run --example live_preview`

use ezviz_openapi::live::VideoQuality;
use ezviz_openapi::EzvizClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_key = std::env::var("EZVIZ_APP_KEY")?;
    let app_secret = std::env::var("EZVIZ_APP_SECRET")?;
    let serial = std::env::var("EZVIZ_DEVICE_SERIAL")?;

    let client = EzvizClient::new(app_key, app_secret);
    let addr = client
        .live_hls_address(&serial, 1, VideoQuality::Hd)
        .await?;
    println!("{}", addr.url);
    Ok(())
}

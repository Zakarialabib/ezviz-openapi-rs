//! Pan a PTZ camera up, then stop it.
//! Run: `EZVIZ_APP_KEY=... EZVIZ_APP_SECRET=... EZVIZ_DEVICE_SERIAL=... cargo run --example ptz_control`

use ezviz_openapi::ptz::{PtzDirection, PtzSpeed};
use ezviz_openapi::EzvizClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_key = std::env::var("EZVIZ_APP_KEY")?;
    let app_secret = std::env::var("EZVIZ_APP_SECRET")?;
    let serial = std::env::var("EZVIZ_DEVICE_SERIAL")?;

    let client = EzvizClient::new(app_key, app_secret);

    client
        .ptz_start(&serial, 1, PtzDirection::Up, PtzSpeed::Medium)
        .await?;
    println!("panning up…");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    client.ptz_stop(&serial, 1, PtzDirection::Up).await?;
    println!("stopped.");

    // Save the current position as preset 1 and recall it.
    let idx = client.preset_add(&serial, 1).await?;
    println!("saved preset index {idx}");
    client.preset_move(&serial, 1, idx).await?;
    println!("recalled preset {idx}");
    Ok(())
}
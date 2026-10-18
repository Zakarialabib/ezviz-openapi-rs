//! List the devices on the EZVIZ account.
//! Run: `EZVIZ_APP_KEY=... EZVIZ_APP_SECRET=... cargo run --example list_devices`

use ezviz_openapi::EzvizClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_key = std::env::var("EZVIZ_APP_KEY")?;
    let app_secret = std::env::var("EZVIZ_APP_SECRET")?;

    let client = EzvizClient::new(app_key, app_secret);
    let list = client.list_devices(0, 50).await?;
    println!("total: {}", list.page.map(|p| p.total).unwrap_or(0));
    for d in &list.items {
        println!(
            "{}  {}  type={}  online={}",
            d.device_serial, d.device_name, d.device_type.as_deref().unwrap_or("-"), d.status
        );
    }
    Ok(())
}
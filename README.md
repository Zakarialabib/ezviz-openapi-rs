# ezviz-openapi

Async Rust client for the **EZVIZ Open Platform** HTTP API
(`open.ys7.com` / `isgpopen.ezvizlife.com`) — the cloud backend behind EZVIZ /
Hikvision smart cameras. Built for the SignageMaster CCTV panel: list devices,
pull HLS/RTMP/FLV live and playback URLs, and drive PTZ cameras (pan/tilt/zoom,
mirror, presets).

Optional `pc-sdk` feature adds **Windows FFI** to the proprietary PCOpenSDK
(`OpenNetStream.dll`) for direct P2P preview and PTZ control from a desktop app.

## Features

- **Auth** — `POST /api/lapp/token/get` with in-memory caching, region
  (`areaDomain`) switching, and one-shot auto-refresh on `10002`.
- **Devices** — list / info / status / camera & channel enumeration.
- **Live & playback** — play addresses (ezopen / HLS / RTMP / FLV), local and
  cloud playback windows, address invalidation.
- **PTZ** — start/stop pan-tilt-zoom (8 directions + zoom + focus), mirror
  flip, presets (add / move / clear).
- **Alarms** — alarm list with typed filters.
- **Capture** — snapshot request.
- **PCOpenSDK FFI** (`pc-sdk`, Windows) — runtime-loaded `OpenNetStream.dll`:
  init, access token, sessions, real-play preview, PTZ control.

## Usage

```toml
[dependencies]
ezviz-openapi = "0.1"
```

```rust,no_run
use ezviz_openapi::{EzvizClient, ptz::{PtzDirection, PtzSpeed}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EzvizClient::new("APP_KEY", "APP_SECRET");

    for d in client.list_devices(0, 50).await?.items {
        println!("{} {}", d.device_serial, d.device_name);
    }

    // HLS live URL for a signage player:
    let live = client.live_hls_address("DEVICE_SERIAL", 1,
        ezviz_openapi::live::VideoQuality::Hd).await?;
    println!("{}", live.url);

    // PTZ:
    client.ptz_start("DEVICE_SERIAL", 1, PtzDirection::Up, PtzSpeed::Medium).await?;
    client.ptz_stop("DEVICE_SERIAL", 1, PtzDirection::Up).await?;
    Ok(())
}
```

See `examples/` for runnable programs (`list_devices`, `ptz_control`,
`live_preview`) driven by `EZVIZ_APP_KEY` / `EZVIZ_APP_SECRET` /
`EZVIZ_DEVICE_SERIAL` env vars.

## PCOpenSDK (Windows, optional)

```toml
[dependencies]
ezviz-openapi = { version = "0.1", features = ["pc-sdk"] }
```

```rust,no_run
use ezviz_openapi::pc_sdk::{PcSdk, PtzCommand, PtzAction};

let sdk = PcSdk::load()?;          // reads $EZVIZ_SDK_DIR, cwd, PATH
sdk.init("APP_KEY")?;
sdk.set_access_token("at.xxxx")?;
let session = sdk.alloc_session_ex(None, std::ptr::null_mut())?;
sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Start, 1)?;
sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Stop, 1)?;
sdk.free_session(&session)?;
sdk.fini_lib()?;
```

The SDK DLL is loaded at runtime and is **not** redistributed by this crate —
place `OpenNetStream.dll` (and its dependency DLLs) on the target machine and
point `EZVIZ_SDK_DIR` at the extracted `EZPCOpenSDK_v5.1.18` folder.

## Development

```bash
cargo build
cargo test          # runs against a local wiremock, no network/credentials
cargo run --example list_devices   # needs EZVIZ_APP_KEY / EZVIZ_APP_SECRET
cargo package --list
cargo publish --dry-run
```

## License

MIT. The EZVIZ PCOpenSDK is proprietary and licensed separately by EZVIZ; this
crate only loads it at runtime and does not include it.

## Reference

`docs/api-reference.md` — full endpoint table, error codes, and a flow diagram
of the API surface.

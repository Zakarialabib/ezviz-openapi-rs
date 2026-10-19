# ezviz-openapi

[![CI](https://github.com/Zakarialabib/ezviz-openapi-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Zakarialabib/ezviz-openapi-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV 1.75](https://img.shields.io/badge/MSRV-1.75-blue.svg)](Cargo.toml)
[![docs.rs](https://img.shields.io/docsrs/ezviz-openapi)](https://docs.rs/ezviz-openapi)

An async Rust client for the **EZVIZ Open Platform** — the HTTP control plane
behind EZVIZ and Hikvision consumer cameras. It mints access tokens, lists
devices and channels, issues and revokes live/playback URLs, and drives PTZ,
presets and alarms. An optional `pc-sdk` feature binds the proprietary Windows
PCOpenSDK for low-latency preview.

**What this crate is not.** It does not decode or render video, and it is not a
player. It returns URLs and commands; playback is the caller's job. The
`ezopen` protocol value can be requested but only the vendor SDK can play it —
use HLS (`live_hls_address`) for anything portable.

## Quick start

```toml
[dependencies]
ezviz-openapi = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use ezviz_openapi::{EzvizClient, ptz::{PtzDirection, PtzSpeed}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EzvizClient::new("APP_KEY", "APP_SECRET"); // token fetched on first call

    for device in client.list_devices(0, 50).await?.items {
        println!("{} {} online={}", device.device_serial, device.device_name, device.status);
    }

    let live = client
        .live_hls_address("DEVICE_SERIAL", 1, ezviz_openapi::live::VideoQuality::Hd)
        .await?;
    println!("play: {} (expires {:?})", live.url, live.expire_time);

    client.ptz_start("DEVICE_SERIAL", 1, PtzDirection::Up, PtzSpeed::Medium).await?;
    client.ptz_stop("DEVICE_SERIAL", 1, PtzDirection::Up).await?;
    Ok(())
}
```

Credentials belong on a server. The crate never logs, persists or transmits
them anywhere except to the EZVIZ host that issued them.

## Feature flags

| Feature | Default | Enables | Cost |
|---|---|---|---|
| `rustls-tls` | yes | TLS through `reqwest/rustls` | pure-Rust TLS stack |
| `pc-sdk` | no | Windows FFI to PCOpenSDK via `libloading` | `libloading` dep; Windows only |

With `pc-sdk` off nothing Windows-specific is compiled, so Linux/macOS builds
and the published crate stay dependency-light.

## What it covers

| Module | Platform surface | Main methods |
|---|---|---|
| `token` | `POST /api/lapp/token/get` | cached automatically; `access_token()`, `refresh_token()` |
| `device` | `/device/list`, `/device/info`, `/device/status`, `/camera/list`, `/device/camera/list` | `list_devices()`, `device_info()`, `device_status()`, `list_cameras()`, `device_cameras()` |
| `live` | `POST /api/lapp/live/address/get`, `.../disable` | `live_address()`, `live_hls_address()`, `disable_live_address()` |
| `playback` | same endpoint with `type=2`/`3` | `local_playback_address()`, `cloud_playback_address()` |
| `ptz` | `/device/ptz/start`, `/stop`, `/mirror`, `/preset/{add,move,clear}` | `ptz_start()`, `ptz_stop()`, `ptz_mirror()`, `preset_add()`, `preset_move()`, `preset_clear()` |
| `alarm` | `POST /api/lapp/alarm/list` | `list_alarms()` |
| `capture` | `POST /api/lapp/device/capture` | `capture()` |
| `pc_sdk` | PCOpenSDK (`OpenNetStream.dll`) | `PcSdk::load()`, `init()`, `start_real_play_ex()`, `ptz_ctrl_ex()`, … |

Full endpoint table, parameter rules and error codes:
[`docs/01-api-reference.md`](docs/01-api-reference.md).

## Windows FFI (`pc-sdk`)

```rust,no_run
use ezviz_openapi::pc_sdk::{PcSdk, PtzAction, PtzCommand};

let sdk = PcSdk::load()?;               // $EZVIZ_SDK_DIR, cwd, then PATH
sdk.init("APP_KEY")?;
sdk.set_access_token("at.xxxx")?;
let session = sdk.alloc_session_ex(None, std::ptr::null_mut())?;
sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Start, 1)?;
sdk.ptz_ctrl_ex(&session, "DEVICE_SERIAL", 1, PtzCommand::Up, PtzAction::Stop, 1)?;
sdk.free_session(&session)?;
sdk.fini_lib()?;
```

The DLL is loaded at runtime from `EZVIZ_SDK_DIR` and is **not** redistributed.
`EZPCOpenSDK_v5.1.18/` is gitignored, absent from the published crate (verified:
`cargo package --list` lists zero SDK files), and no build step reads it. Two
pieces are kept in the working tree for provenance and manual testing: `inc/`
(the headers the bindings were transcribed from) and `lib/win64/` (the runtime
DLLs).

## Examples

| Example | Needs | Shows |
|---|---|---|
| `list_devices` | `EZVIZ_APP_KEY`, `EZVIZ_APP_SECRET` | token minting + device listing |
| `live_preview` | + `EZVIZ_DEVICE_SERIAL` | HLS live URL |
| `ptz_control` | + `EZVIZ_DEVICE_SERIAL` | pan up/stop via PTZ |

```bash
cargo run --example list_devices
```

## Integrate it

Nine copy-paste prompts for embedding this crate — live HLS widget, token
broker, PTZ panel, alarm poller, snapshot tile, NVR grid, playback scrubber,
Windows P2P preview, and credential-free tests — live in
[`docs/04-integration-prompts.md`](docs/04-integration-prompts.md). Each names
the exact APIs, the pitfalls that bite (7-day alarm window, fire-and-forget
capture, no RTSP in the API), and what "done" means.

The shortest useful one:

```text
Add a live EZVIZ camera view using ezviz-openapi. Resolve the URL with
client.live_hls_address(device_serial, channel_no, VideoQuality::Fluent) for
players, cache it until expire_time, and re-resolve instead of treating an
expired URL as an offline camera. Release torn-down streams with
disable_live_address(serial, channel, &id). Credentials stay server-side: the
frontend receives a URL, never an appKey. Done when the widget plays from a cold
start, survives expiry without a black flash, and a closed widget releases its
address.
```

Whichever scenario you build, the platform rules are the same: tokens are per
appKey **per region** (not per player), `areaDomain` overrides your configured
host, and the crate returns URLs — it never decodes video.

## Development

```bash
rustup component add rustfmt clippy   # CI enforces both

cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                            # wiremock; no network, no credentials
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

`cargo test --features pc-sdk` on Windows covers the FFI module; elsewhere the
module compiles to an `Unsupported` error and its tests are skipped.

`master` is protected: all five CI checks (rustfmt, clippy, rustdoc, tests on
Linux and Windows) must pass and force-pushes are rejected.

## Releasing

```bash
bash scripts/release.sh --dry-run        # gates + package + verify, no upload
bash scripts/release.sh --version 0.2.0  # bump and publish
```

`release.yml` calls the same script on a `v*.*.*` tag, so the tag and the script
cannot disagree about the gates.

## Known limitations

- **Verify-code error codes are unconfirmed.** `400035`/`400036` are
  mobile-SDK codes; the HTTP-path equivalents and whether encrypted HLS needs
  the `code` parameter must be confirmed against real hardware.
- **`capture()` is fire-and-forget.** It requests a snapshot; the image is
  fetched later from the platform's picture URL.
- **Alarms are capped at 7 days** and paginated; a long outage needs a cursor
  and paging, not a single query.
- **No WebSocket/MQTT push.** Alarm delivery is polling-only here; a broker
  layer is the consumer's responsibility (see
  [`docs/02-cctv-integration.md`](docs/02-cctv-integration.md)).

## Project links

- [CHANGELOG.md](CHANGELOG.md) — release history, limitations
- [CONTRIBUTING.md](CONTRIBUTING.md) — gates, design rules, commit convention
- [SECURITY.md](SECURITY.md) — private disclosure, in-scope surfaces
- [docs/00-index.md](docs/00-index.md) — documentation map

## License

MIT — see [LICENSE](LICENSE). The EZVIZ PCOpenSDK is proprietary and licensed
separately by EZVIZ; this crate only loads it at runtime.

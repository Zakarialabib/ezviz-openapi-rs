# ezviz-openapi

[![CI](https://github.com/Zakarialabib/ezviz-openapi-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Zakarialabib/ezviz-openapi-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV 1.75](https://img.shields.io/badge/MSRV-1.75-blue.svg)](Cargo.toml)
[![docs.rs](https://img.shields.io/docsrs/ezviz-openapi)](https://docs.rs/ezviz-openapi)

An async Rust client for the **EZVIZ Open Platform** — the HTTP control plane
behind EZVIZ and Hikvision consumer cameras. Mint tokens, list devices and
channels, issue and revoke live/playback URLs, drive PTZ and presets, read
alarms, and request snapshots. An optional `pc-sdk` feature binds the
proprietary Windows PCOpenSDK for low-latency preview.

```rust
use ezviz_openapi::{EzvizClient, ptz::{PtzDirection, PtzSpeed}};

let client = EzvizClient::new("APP_KEY", "APP_SECRET");   // token minted on first call
let live = client.live_hls_address("CAMERA_SERIAL", 1,
    ezviz_openapi::live::VideoQuality::Hd).await?;
println!("{}", live.url);

client.ptz_start("CAMERA_SERIAL", 1, PtzDirection::Up, PtzSpeed::Medium).await?;
client.ptz_stop("CAMERA_SERIAL", 1, PtzDirection::Up).await?;
```

## Why this exists

The EZVIZ API is a form-encoded JSON API with three sharp edges: every response
is wrapped in a `{code, msg, data, page}` envelope whose `code` is sometimes a
string and sometimes a number; tokens are scoped to an app key **and a region**,
and the region only arrives in the token response; and a long list of endpoints
are pure acknowledgements with no `data` at all. Wrapping it once, correctly,
means an application never has to rediscover those rules.

The rules the code follows:

- **One error type.** A platform rejection preserves the platform's own `code`
  and `msg` (`EzvizError::Api`) instead of being flattened into a string.
- **Regions are followed, not configured.** `areaDomain` from the token response
  replaces the base URL if it differs from the default host.
- **Token expiry is handled once.** A call that comes back `10002` is refreshed
  and replayed exactly once, inside the client.
- **Nothing panics.** Every fallible path returns `EzvizError`; no `unwrap()` in
  library code, lock poisoning included.

## What it covers

| Area | Endpoints | Methods |
|---|---|---|
| Tokens | `POST /api/lapp/token/get` | `access_token()` (cached 24 h), `refresh_token()` |
| Devices | `/device/list`, `/device/info`, `/device/status`, `/camera/list`, `/device/camera/list` | `list_devices()`, `device_info()`, `device_status()`, `list_cameras()`, `device_cameras()` |
| Live | `POST /api/lapp/live/address/get`, `.../disable` | `live_address()`, `live_hls_address()`, `disable_live_address()` |
| Playback | same endpoint, `type=2/3` + window | `local_playback_address()`, `cloud_playback_address()` |
| PTZ | `/device/ptz/{start,stop,mirror}`, `/device/preset/{add,move,clear}` | `ptz_start()`, `ptz_stop()`, `ptz_mirror()`, `preset_add()`, `preset_move()`, `preset_clear()` |
| Alarms | `POST /api/lapp/alarm/list` | `list_alarms()` |
| Snapshots | `POST /api/lapp/device/capture` | `capture()` |
| Windows SDK | PCOpenSDK (`OpenNetStream.dll`) | `PcSdk::load()`, `init()`, `start_real_play_ex()`, `ptz_ctrl_ex()`, … |

Full parameter rules, error codes and the FFI surface:
[`docs/01-api-reference.md`](docs/01-api-reference.md).

## What it does not do

It does not decode video, render it, or store recordings. It returns URLs and
issues commands — playback belongs to whatever fetches those URLs. `ezopen` can
be requested but only vendor SDKs can play it; use HLS for anything portable.
There is no WebSocket/event push either: alarms are polled.

## When to use it — and when not to

| Use it when | Skip it when |
|---|---|
| You control EZVIZ/Hikvision cameras from a server, desktop app or service | You only need a single snapshot — a plain HTTP call may do |
| You need PTZ, presets, alarm history or cloud playback windows | You need RTSP only: EZVIZ's API has no RTSP value, build the URL yourself |
| You want tokens and regions handled for you | You are on Windows **and** need sub-second latency — use the `pc-sdk` feature, not the HTTP path |
| You are behind a broker that must keep the app secret server-side | You need on-device playback: that is a player, not this crate |

## Install

```toml
[dependencies]
ezviz-openapi = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Windows FFI is opt-in:

```toml
ezviz-openapi = { version = "0.1", features = ["pc-sdk"] }
```

| Feature | Default | Enables | Cost |
|---|---|---|---|
| `rustls-tls` | yes | TLS through `reqwest/rustls` | pure-Rust TLS stack |
| `pc-sdk` | no | PCOpenSDK FFI via `libloading` | Windows only; adds one dependency |

## Where things live

| Path | What |
|---|---|
| `src/client.rs` | `EzvizClient`, the envelope handling, the `10002` replay |
| `src/token.rs` | token response and `areaDomain` |
| `src/device.rs`, `src/live.rs`, `src/playback.rs`, `src/ptz.rs`, `src/alarm.rs`, `src/capture.rs` | one module per endpoint family |
| `src/pc_sdk.rs` | the Windows FFI surface (`pc-sdk` only) |
| `examples/` | three runnable programs |
| `docs/` | reference, use cases, glossary |

## How it is built

```
your code ──▶ EzvizClient ──▶ reqwest (rustls) ──▶ EZVIZ API
                    │
                    ├── token cache (24 h) + one replay on 10002
                    ├── areaDomain → base URL
                    └── EzvizError { Api, Http, Json, Io, Sdk }
```

Everything that can be tested without a network is a free function: envelope
parsing, the flexible `ApiCode`, key/path formatting. The test suite runs against
`wiremock`, so CI needs no account and no camera.

## Windows FFI (`pc-sdk`)

`PcSdk::load()` searches `$EZVIZ_SDK_DIR`, then the working directory, then
`PATH`, for `OpenNetStream.dll`. Nothing is redistributed: the DLL is loaded at
runtime and the SDK folder is gitignored (verified — `cargo package --list`
contains zero SDK files). `inc/` and `lib/win64/` stay in the working tree only
so the bindings remain auditable against their headers.

## Examples

| Example | Needs | Shows |
|---|---|---|
| `list_devices` | `EZVIZ_APP_KEY`, `EZVIZ_APP_SECRET` | token minting + device listing |
| `live_preview` | + `EZVIZ_DEVICE_SERIAL` | HLS live URL |
| `ptz_control` | + `EZVIZ_DEVICE_SERIAL` | pan and stop |

```bash
cargo run --example list_devices
```

## Development

```bash
rustup component add rustfmt clippy   # CI enforces both

cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                            # wiremock; no network, no credentials
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

bash scripts/release.sh --dry-run     # the full release preflight, no upload
```

`cargo test --features pc-sdk` on Windows covers the FFI module; elsewhere that
module compiles to an `Unsupported` error and its tests are skipped. `master` is
protected: the five CI checks must pass and force-pushes are rejected.

## Documentation

Start at [`docs/glossary.md`](docs/glossary.md) — it is the map: every term, and
the page or source file that explains it.

| Page | Answers |
|---|---|
| [`docs/00-index.md`](docs/00-index.md) | reading order and conventions |
| [`docs/01-api-reference.md`](docs/01-api-reference.md) | endpoints, parameters, error codes, FFI surface |
| [`docs/02-use-cases.md`](docs/02-use-cases.md) | live wall, alarm pipeline, broker, snapshots, playback, desktop preview |
| [`docs/03-external-refs.md`](docs/03-external-refs.md) | official docs and the SDK |
| [`docs/04-integration-prompts.md`](docs/04-integration-prompts.md) | copy-paste prompts per scenario |
| [`docs/glossary.md`](docs/glossary.md) | terminology and navigation |

Also: [`CHANGELOG.md`](CHANGELOG.md), [`CONTRIBUTING.md`](CONTRIBUTING.md),
[`SECURITY.md`](SECURITY.md).

## Known limitations

- **Verify-code error codes are unconfirmed.** `400035`/`400036` are
  mobile-SDK codes; the HTTP-path equivalents and whether encrypted HLS needs the
  `code` parameter must be confirmed on real hardware.
- **`capture()` is fire-and-forget.** It requests a snapshot; the image is
  fetched afterwards from the platform's picture URL.
- **Alarm queries are capped at 7 days** and paginated — a long outage needs a
  cursor, not one wide query.
- **Polling only.** No WebSocket/MQTT push.

## License

MIT — see [LICENSE](LICENSE). The EZVIZ PCOpenSDK is proprietary and licensed
separately by EZVIZ; this crate only loads it at runtime.

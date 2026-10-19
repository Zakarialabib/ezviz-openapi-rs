# EZVIZ CCTV Integration — Signage Platform

How the `ezviz-openapi` crate is consumed by the signage platform: a live-camera
widget, alarm-driven content interrupt, and a best-effort snapshot widget.

> **Scope.** This is the only document in this repository that describes a
> *consumer* rather than the crate. The platform-side specs it extends
> (SPEC-001 widgets, SPEC-003 DataBroker, SPEC-005 `BindAlarmTrigger`/`PlayerControl`,
> SPEC-006 playlist, SPEC-007 fallback/interrupt, SPEC-008 assets),
> `SIGNAGE_SPEC_PACK_v1.md`, and the `research/` items S1–S5 all live in the
> **signage platform repository**, not here. References to them are external and
> intentionally unresolved from this repo.
>
> **Crate-side reference:** the HTTP surface, error codes and PCOpenSDK bindings
> implemented by `ezviz-openapi` are in [`01-api-reference.md`](01-api-reference.md).

| Status | Item |
|---|---|
| Decided | Decode path (Q1), alarm delivery (Q3), snapshot shape (Q5) |
| Deferred | Broker topology — Phase 2 (Q2); LAN discovery — Phase 3 (Q4) |

---

## 1. Purpose & Scope

Fold CCTV into the signage platform as three capabilities — **not** a separate app:

1. **Live-camera widget** — a `WidgetInstance` rendering inside templates (SPEC-001).
2. **Alarm-driven content interrupt** — EZVIZ alarm → player switches to that
   camera full-screen → dwell → resume playlist. Plugs into playlist `priority`.
3. **Snapshot widget** — periodic JPEG refresh. Best-effort: the `capture`
   endpoint is undocumented, so graceful degradation is required.

**Out of scope:** two-way audio (no v1 signage use case), Wi-Fi provisioning/AP
config, cloud video download, device add/activation (administered in the EZVIZ app).

---

## 2. Provider Abstraction (vendor boundary)

The platform never sees EZVIZ types above this trait:

```rust
// PLATFORM-SIDE trait (lives in the signage repo, not in this crate).
// The crate exposes `EzvizClient`; `EzvizProvider` adapts it to this shape.
#[async_trait]
pub trait CameraProvider: Send + Sync {
    // Streams
    async fn live_url(&self, cam: &CameraRef, proto: StreamProtocol,
                      quality: VideoQuality) -> Result<StreamUrl, CameraError>;
    // Snapshot: returns a URL the caller fetches. `capture` is fire-and-forget;
    // the picture arrives asynchronously and must be fetched from the returned
    // or `alarmPicUrl` URL. The caller owns fetching and caching.
    async fn snapshot_url(&self, cam: &CameraRef) -> Result<Option<String>, CameraError>;
    // Alarms
    async fn poll_alarms(&self, since: SystemTime) -> Result<Vec<Alarm>, CameraError>;
    // PTZ (admin/editor only, never player)
    async fn ptz_start(&self, cam: &CameraRef, dir: PtzDirection, speed: PtzSpeed) -> Result<(), CameraError>;
    async fn ptz_stop(&self, cam: &CameraRef, dir: PtzDirection) -> Result<(), CameraError>;
}

pub struct CameraRef { pub provider: String /* "ezviz" */, pub device_serial: String, pub channel_no: u8 }

pub enum StreamProtocol { Hls, Rtmp, Flv, RtspLocal }  // platform enum, NOT the crate's
```

**Trait → crate mapping (verified against `src/`).** The trait above is the
platform's vocabulary; none of its types are exported by this crate. The adapter
is a thin translation:

| Trait method | Crate call (`EzvizClient`) | Notes |
|---|---|---|
| `live_url(cam, proto, quality)` | `live_address(serial, ch, StreamProtocol, VideoQuality, LiveType, expire, safe_code, start, stop)` | Use `live_hls_address(serial, ch, quality)` for the P1 default. `LiveType::Preview` for live, `LocalPlayback`/`CloudPlayback` + `startTime`/`stopTime` (`yyyy-MM-dd HH:mm:ss`) for playback. |
| `snapshot_url(cam)` | `capture(serial, ch)` | Returns `Result<()>` only — **no URL**. The platform derives the picture URL from alarm/device picture fields. |
| `poll_alarms(since)` | `list_alarms(start_ms, end_ms, AlarmType, AlarmStatus, page_start, page_size)` | Timestamps are **Unix milliseconds**, and only the **last 7 days** are queryable — `since` older than 7 days must be clamped. Paginated: loop until `page` is exhausted. |
| `ptz_start(cam, dir, speed)` | `ptz_start(serial, ch, PtzDirection, PtzSpeed)` | Explicit `ptz_stop` is required; the platform does not auto-stop. |
| `ptz_stop(cam, dir)` | `ptz_stop(serial, ch, PtzDirection)` | Same direction as the matching start. |

Type mapping: `CameraRef` → `(device_serial: &str, channel_no: i32)`;
`Alarm` → `AlarmItem`; `CameraError` → `EzvizError`; crate
`StreamProtocol` = `{ Ezopen, Hls, Rtmp, Flv }` — **it has no `RtspLocal`
variant**, so the platform's `RtspLocal` never reaches the HTTP API (see §3 P2).


- `EzvizProvider` wraps `ezviz-openapi` (HTTP) — impl #1. Hikvision ISAPI / ONVIF
  become impl #2+ later without touching SPEC-006/007.
- The HTTP/SDK direction-code overlap (they match for 0–11) stays an internal
  detail of `EzvizProvider`; the trait uses its own types. See
  [`01-api-reference.md`](01-api-reference.md) §PCOpenSDK — the HTTP API is the
  portable path, the SDK is the low-latency desktop path.
- `snapshot_url` returns a URL or `None`, **never `Bytes`** — the fetch is async
  and belongs to the caller.

---

## 3. Streaming Paths (three tiers, one widget)

| Tier | Path | Latency | Used by | Notes |
|---|---|---|---|---|
| **P1 Player (cloud)** | HTTP `live/address/get` → **HLS** URL → GStreamer `souphttpsrc ! hlsdemux` | 5–15 s tuned | Player runtime | Default. RTMP/FLV per device-class matrix where HLS demux is weak. Tune `gst-hls` (`live=true`, `target-duration`); 10–30 s is the untuned worst case. |
| **P2 Player (LAN)** | Local **RTSP** `rtsp://<cam-ip>:554/h264/ch1/<ch>/av_stream` (verification-code auth) | <1 s | Player on the cameras' LAN | Requires RTSP enabled in the EZVIZ app; not all models support it. **The crate cannot mint this URL** — `StreamProtocol` has no RTSP variant, so the URL is constructed platform-side from the LAN address. The compiler chooses P2 when `CameraRef` carries a LAN address and the probe succeeds. LAN address population is manual admin entry for now (Q4). |
| **P3 Admin (desktop)** | PCOpenSDK `OpenSDK_StartRealPlayEx` P2P into HWND (`pc-sdk` feature) | <1 s | Tauri editor/admin, **Windows only** | HWND embedding is incompatible with the player's wgpu compositor — admin preview only. |

**Decode → compositor (Q1).** Camera frames reach the wgpu compositor via
**GStreamer decode → texture upload (D3D11 on Windows / DMA-BUF on Linux) → wgpu
texture**. This keeps cameras as true compositor widgets (z-order, transforms,
multi-camera templates). Per-camera overlay GL windows and libmpv are fallbacks
only if texture sharing proves unworkable — revisit on evidence, not by default.

Rules:

- **Players never load `OpenNetStream.dll` and never hold app credentials.** P3 is
  editor-side only; the proprietary SDK is never redistributed.
- The playlist stores `stream: { kind: "ezviz_hls", camera: …, lan_fallback: <rtsp-url?> }`
  and the player resolves at runtime: try P2 if configured and reachable, else P1 via broker.

---

## 4. Token Broker Service (backend, new component)

Players must not ship appKey/appSecret. Server-mode flow per the EZVIZ help
center: the server obtains `accessToken` + expiry and distributes it; clients call
`setAccessToken`.

```rust
// camera-broker service (part of compile/deploy backend)
- holds appKey/appSecret (per-region: isgpopen intl / areaDomain / ys7 China)
- mints short-lived tokens and distributes them to players
- renews on 10002 (token expired) with one-shot auto-refresh (crate behavior)
- rate-limit aware: caches resolved live URLs per camera (explicit expireTime,
  e.g. 24 h), never per-player fan-out (10007/10029 protection at fleet scale)
- player → broker: mTLS or device token; broker → EZVIZ: app credentials
```

**Token scoping (corrected).** EZVIZ tokens are **per-appKey/region, not
per-player** — the platform has no per-player token scoping. The real scoping
mechanism is the **live-URL `id`** (revocable via `/api/lapp/live/address/disable`)
plus rate limits. The broker mints one token per appKey/region and scopes *URLs*
per camera, not tokens per player.

**Broker topology (Q2, deferred to Phase 2).** Default to evaluate: a **Rust
sidecar** wrapping the tested `ezviz-openapi` crate, exposed as a local HTTP
service that Laravel proxies — reuses the tested token/URL/error logic and avoids
PHP↔Rust protocol drift. A pure-PHP reimplementation is the fallback if the broker
must live entirely in Laravel; accept the drift cost explicitly.

Playlist auth block:

```json
"auth": { "kind": "broker_token", "endpoint": "https://broker…/ezviz/token",
          "refresh_secs": 4200 }
```

---

## 5. Playlist Additions (SPEC-006 delta)

```jsonc
// live camera widget
{ "type": "live_stream", "camera": { "provider": "ezviz", "device_serial": "…", "channel_no": 1 },
  "protocol_pref": ["rtsp_lan", "hls"],
  "quality": "fluent",               // HD for admin, fluent for players (bandwidth)
  "auth": { "kind": "broker_token", "refresh_secs": 4200 },
  "rect": [100, 200, 800, 450],
  "on_stream_lost": { "action": "fallback_asset", "asset": "offline_card.png" },
  "interruptible": true }

// snapshot widget (best-effort)
{ "type": "camera_snapshot", "camera": { "…": "…" },
  "refresh_secs": 60, "on_failure": "hide" }

// alarm interrupt binding (SPEC-005 command: BindAlarmTrigger)
{ "type": "alarm_trigger", "camera": { "…": "…" }, "alarm_types": ["motion"],
  "action": { "type": "live_stream_fullscreen", "dwell_secs": 30, "then": "resume" } }
```

Player semantics: alarm event → swap to the bound `live_stream` at interrupt
priority → dwell → resume playlist. Camera capability flags (`isSupportTalk`, PTZ
support levels) are admin-only metadata; players ignore them.

**Alarm delivery (Q3, decided).** The **broker polls EZVIZ once per camera**
(`/api/lapp/alarm/list`) and **pushes events to players** over WebSocket/SSE.
Players never poll EZVIZ directly. Per-player polling is rejected — it means N
pollers per camera at fleet scale. EZVIZ's MQTT Message System is the scale path
to evaluate later.

Polling constraints that shape the broker loop (verified in `src/alarm.rs`):
timestamps are **Unix milliseconds**, the query window is **at most 7 days**, and
the call is **paginated** (`pageStart`/`pageSize`). The broker therefore keeps a
durable cursor per camera, clamps the window to 7 days on a cold start, and pages
through results rather than assuming one page. `AlarmType`/`AlarmStatus` filters
are enum-valued, so the "all types" case is a real enum value, not an empty
string.

---

## 6. Editor / Admin Integration

- **CCTV widget in editor canvas (Phase 2):** live preview via broker (P1 HLS in a
  video element is fine for preview); PTZ panel via `CameraProvider::ptz_*`.
- **Windows admin build only:** P3 low-latency preview via the `pc-sdk` feature
  (HWND overlay window). Feature-gated; Linux/macOS admin uses P1.
- **Verify-code / stream encryption UX:** if the device has encryption enabled
  (`isEncrypt`), the stream needs the 6-character verification code. The
  `400035`/`400036` codes are **mobile-SDK** codes — the HTTP-path verify-code
  error codes must be confirmed empirically, and encrypted-device HLS may require
  the `code` param in `live/address/get`. On error, prompt and retry.
- **Video definition (`videoLevel`):** set before or during preview; takes effect
  on a stop→start cycle (SDK-documented behaviour — do not expect hot-swap).

---

## 7. Security & Compliance

1. No appKey/appSecret on any player device; broker-only issuance.
2. Verification codes stored server-side, never in playlists; delivered to players
   over the device-auth channel only when a P2 RTSP path requires them.
3. The proprietary SDK is never committed or redistributed; `pc-sdk` loads it at
   runtime via `libloading` (crate invariant). `EZPCOpenSDK_v5.1.18/` is gitignored.
4. Privacy: camera feeds in public signage need jurisdictional review (retention,
   signage of CCTV operation). Flagged per-deployment, not per-template.
5. REA guardrails apply (see [`02-external-refs.md`](02-external-refs.md)): REA is
   used only when SDK/docs behaviour diverges from documented behaviour.

---

## 8. Error Model (unified)

| Source | Code | Meaning | Platform behavior |
|---|---|---|---|
| HTTP | 10002 | token expired | broker auto-refresh once, retry; else player holds last frame + fallback card |
| HTTP | 10005/10007/10029 | frozen / rate-limited | broker backoff + jitter; player unaffected (cached URL) |
| HTTP | 20002/20007/20018 | device missing/offline/illegal | widget → `on_stream_lost` fallback |
| HTTP | 60000–60011 | PTZ unsupported/limit/fail | admin UI toast; never player-visible |
| SDK (P3) | 46/47/19 | PTZ ok / failed / exception | admin preview message callback (`src/pc_sdk.rs`) |
| Stream | verify-code (HTTP codes TBD) | code required/mismatch | admin re-prompt flow; player → fallback |
| Player | any | stream stall > 5 s | `on_stream_lost` fallback; alarm to fleet monitor |

Full HTTP error-code table: [`01-api-reference.md`](01-api-reference.md).

---

## 9. Acceptance Criteria

- [ ] Player renders an HLS camera widget at 1080p alongside playlist video;
      stream kill → fallback card ≤ 5 s; recovery auto within 2 refresh cycles.
- [ ] P1 glass-to-glass ≤ 15 s (tuned HLS). LAN path: player on the camera VLAN
      plays RTSP (P2) at < 1 s glass-to-glass; cloud fallback engages when the
      RTSP probe fails.
- [ ] Alarm interrupt: injected alarm → fullscreen camera ≤ 3 s end-to-end;
      dwell → playlist resumes at the correct item boundary.
- [ ] 500 players share one camera: broker resolves 1 upstream URL (cached),
      ≤ 1 token refresh/hour/player, zero 10007/10029 errors in soak.
- [ ] No `OpenNetStream.dll`, appKey, or appSecret present in any player artifact.
      **Not implemented yet:** `.github/workflows/release.yml` runs `cargo check` +
      `cargo publish` only — no artifact scan and no `cargo test`. The lint must be
      built before this criterion can pass.
- [ ] PTZ from the Windows admin build verified against a live camera; Linux/macOS
      admin degrades to P1 preview with PTZ via HTTP.

---

## 10. Phasing & Traceability

| Phase | Work | Gates |
|---|---|---|
| 2 (editor) | CCTV widget, broker MVP (**Q2 decided here**), admin PTZ panel, verify-code UX | AC §9 (admin-side) |
| 3 (player) | GStreamer HLS/RTSP paths (**Q1 texture-upload spike first**), interrupt engine, alarm poller (**Q3 broker-push**), fallback matrix, **Q4 decided here** | AC §9 (player-side) + 48 h soak with simulated alarms |
| 4 | Multi-provider (Hikvision ISAPI/ONVIF) behind `CameraProvider`; fleet camera analytics | — |

**External traceability:** extends SPEC-001 (WidgetInstance), SPEC-003 (DataBroker
← alarm/URL sources), SPEC-005 (`BindAlarmTrigger`, `PlayerControl`), SPEC-006
(playlist items above), SPEC-007 (fallback/interrupt/watchdog interplay), SPEC-008
(snapshot assets via content-hash store). Research items S1–S5 inform §3/§5/§8.
All of these live in the signage platform repository.

---

## 11. Decision Log

Decisions and factual corrections from the review round of 2026-10-17. The body
above reflects the *corrected* state; this table records what changed and why.

| ID | Topic | Verdict |
|---|---|---|
| Q1 | Decode → compositor path | GStreamer decode → texture upload (D3D11 / DMA-BUF) → wgpu texture. Overlay windows and libmpv are fallbacks only on evidence. |
| Q2 | Broker topology | **Deferred to Phase 2.** Default to evaluate: Rust sidecar wrapping `ezviz-openapi`, Laravel proxies. |
| Q3 | Alarm delivery | Broker polls EZVIZ once per camera and pushes to players (WebSocket/SSE). MQTT Message System is the later scale path. |
| Q4 | Camera LAN discovery (P2) | **Deferred to Phase 3.** Default: manual admin entry; mDNS/UPnP later. |
| Q5 | Snapshot trait shape | `snapshot_url()` returns a URL; the caller fetches. Never synchronous `Bytes`. |
| C1 | Token scoping | Tokens are per-appKey/region, not per-player. Scoping = live-URL `id` (revocable) + rate limits. |
| C2 | Verify-code codes | `400035`/`400036` are mobile-SDK codes; the HTTP-path codes and the `code` param for encrypted HLS must be confirmed empirically. |
| C3 | Snapshot semantics | `capture` is fire-and-forget; the picture arrives asynchronously. |
| C4 | Alarm scaling | Per-player polling rejected in favour of broker-push (Q3). |
| C5 | HLS latency | 10–30 s is the untuned worst case; tune `gst-hls` to 5–15 s. |

---

## Source reconciliation (2026-10-18)

Every structural claim about the crate was probed against `src/`. This section
records what was wrong and what was confirmed.

| Claim (doc, before) | Verified reality | Evidence |
|---|---|---|
| §2 trait used `CameraRef`, `Alarm`, `CameraError` as if they were crate types | None of those exist here. The crate exports `EzvizClient`, `AlarmItem`, `EzvizError`; device identity is `(device_serial: &str, channel_no: i32)` parameters, not a `CameraRef` struct | `grep -rhoE 'pub (struct\|enum) [A-Za-z]+' src/*.rs` |
| §2 `StreamProtocol { Hls, Rtmp, Flv, RtspLocal }` | The crate's `StreamProtocol` is `{ Ezopen, Hls, Rtmp, Flv }` — **no RTSP variant**. RTSP is a platform-built LAN URL, never an API request | `src/live.rs:11-20` |
| §2 `poll_alarms(since: SystemTime)` | Crate call is `list_alarms(start_ms, end_ms, AlarmType, AlarmStatus, page_start, page_size)`: Unix **milliseconds**, **7-day maximum window**, paginated | `src/alarm.rs:116-130` |
| §2 `snapshot_url` implies the call yields a URL | `capture()` returns `Result<()>` only; no URL is returned. The picture URL must come from alarm/device picture fields | `src/capture.rs:12-25` |
| §9 "release lint: Rust-binary scan + string grep in CI" | No such lint exists. `release.yml` runs `cargo check` + `cargo publish` only — no test step, no artifact scan | `.github/workflows/release.yml:16-23` |
| §3 P2 RTSP row implied the API could supply the URL | Corrected in place: the URL is constructed platform-side | `src/live.rs:11-20` |

Confirmed clean (no edit needed):

| Claim | Evidence |
|---|---|
| One-shot auto-refresh on `10002` | `src/client.rs:153-166` — refresh once, replay the request |
| `isEncrypt` and `videoLevel` are modelled | `src/device.rs:31,60,95,99` |
| `EZPCOpenSDK_v5.1.18/` is gitignored | `.gitignore:2` |
| `OpenSDK_StartRealPlayEx` is the P3 entry point | `src/pc_sdk.rs:384` (`sym("OpenSDK_StartRealPlayEx")`) |
| `live/address/get`, `live/address/disable`, `alarm/list`, `device/capture`, `ptz/start`, `ptz/stop` are implemented | `src/live.rs`, `src/alarm.rs`, `src/capture.rs`, `src/ptz.rs` |
| `400035`/`400036` are withheld as mobile-SDK codes | absent from `src/` — the doc already marks them unconfirmed (C2) |

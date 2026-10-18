# SPEC-011 — CCTV / Camera Integration (EZVIZ + provider-generic) — v2

> Companion to `SIGNAGE_SPEC_PACK_v1.md`. Implements the direction set in the
> EZVIZ integration decision: in-house `ezviz-openapi` crate is the primary
> integration; REA is fallback-only for undocumented behavior.
>
> **v2 changes** (grill-me round, 2026-10-17): decisions Q1–Q5 recorded in
> §11; factual corrections C1–C5 applied inline (token scoping, verify-code
> codes, snapshot semantics, alarm delivery, HLS latency).
>
> **References:**
> - PC SDK manual (legacy, deprecated): https://open.ys7.com/doc/en/pc/ — changelog only
>   (V3.0.0 2017 → V4.2.0 2018); docs center is being sunset ("此文档即将下线").
> - Current help center: https://open.ys7.com/help/en/ (iOS = /491, Android = /492,
>   PC SDK section under construction). PC SDK headers: `EZPCOpenSDK_v5.1.18/inc/`.
> - HTTP API: `open.ys7.com/help/en/*` (implemented by `ezviz-openapi`, see its
>   `docs/api-reference.md`).
> - Local-stream precedent: RTSP enabled manually in EZVIZ app, URL pattern
>   `rtsp://<ip>:554/h264/ch1/main/av_stream`, auth `admin` / 6-char verification code.

---

## 1. Purpose & Scope

Fold CCTV into the signage platform as three capabilities — **not** a separate app:

1. **Live-camera widget** — a `WidgetInstance` rendering inside templates (SPEC-001).
2. **Alarm-driven content interrupt** — EZVIZ alarm → player switches to that camera
   full-screen → dwell → resume playlist. (Differentiator; plugs into playlist
   `priority` from research item S5.)
3. **Snapshot widget** — periodic JPEG refresh (best-effort; `capture` endpoint is
   undocumented — graceful degradation required).

Out of scope: two-way audio (no signage use case v1), Wi-Fi provisioning/AP config,
cloud video download, device add/activation flows (administered via EZVIZ app).

---

## 2. Provider Abstraction (vendor boundary)

The platform never sees EZVIZ types above this trait:

```rust
#[async_trait]
pub trait CameraProvider: Send + Sync {
    // Streams
    async fn live_url(&self, cam: &CameraRef, proto: StreamProtocol,
                      quality: VideoQuality) -> Result<StreamUrl, CameraError>;
    // Snapshot: returns a URL the caller fetches (capture is fire-and-forget;
    // the picture is delivered asynchronously — see C3/Q5).
    async fn snapshot_url(&self, cam: &CameraRef) -> Result<Option<String>, CameraError>;
    // Alarms
    async fn poll_alarms(&self, since: SystemTime) -> Result<Vec<Alarm>, CameraError>;
    // PTZ (admin/editor only, never player)
    async fn ptz_start(&self, cam: &CameraRef, dir: PtzDirection, speed: PtzSpeed) -> Result<(), CameraError>;
    async fn ptz_stop(&self, cam: &CameraRef, dir: PtzDirection) -> Result<(), CameraError>;
}

pub struct CameraRef { pub provider: String /* "ezviz" */, pub device_serial: String, pub channel_no: u8 }

pub enum StreamProtocol { Hls, Rtmp, Flv, RtspLocal }  // ezopen: excluded (see §6)
```

- `EzvizProvider` wraps `ezviz-openapi` (HTTP) — impl #1. Hikvision ISAPI / ONVIF
  become impl #2+ later without touching SPEC-006/007.
- HTTP/SDK enum overlap (direction codes 0–11 "happen to match") stays an internal
  detail of `EzvizProvider`; the trait uses its own types.
- **C3/Q5:** `snapshot` returns a URL (or `None`), never `Bytes` — the EZVIZ
  `capture` endpoint is fire-and-forget; the picture arrives asynchronously and
  must be fetched from the returned/`alarmPicUrl` URL. The caller owns the fetch
  and caching.

---

## 3. Streaming Paths (three tiers, one widget)

| Tier | Path | Latency | Used by | Notes |
|---|---|---|---|---|
| **P1 Player (cloud)** | HTTP `live/address/get` → **HLS** URL → GStreamer `souphttpsrc ! hlsdemux` | 5–15 s (tuned) | Player runtime | Default. RTMP/FLV per device-class matrix where HLS demux is weak. **C5:** tune `gst-hls` (`live=true`, `target-duration`) to hold latency at the low end; 10–30 s is the untuned worst case. |
| **P2 Player (LAN)** | Local **RTSP** `rtsp://<cam-ip>:554/h264/ch1/<ch>/av_stream` (verification-code auth) | <1 s | Player on same LAN as cameras | Requires RTSP enabled in EZVIZ app; not all models support it. Compiler chooses P2 when `CameraRef` carries a LAN address + probe succeeds. **Q4 (deferred to Phase 3):** LAN address population — default is manual admin entry; mDNS/UPnP scan evaluated later. |
| **P3 Admin (desktop)** | PCOpenSDK `OpenSDK_StartRealPlayEx` P2P into HWND (`pc-sdk` feature) | <1 s | Tauri editor/admin, **Windows only** | HWND embedding — incompatible with player wgpu compositor; admin preview only. |

**Q1 (decided): decode → compositor path.** Camera frames reach the wgpu
compositor via **GStreamer decode → texture upload (D3D11 on Windows / DMA-BUF
on Linux) → wgpu texture**. This keeps cameras as true compositor widgets
(z-order, transforms, multi-camera templates). Per-camera overlay GL windows
and libmpv are the fallbacks if texture sharing proves unworkable — revisit
only on evidence, not by default.

Rules:
- **Players never load `OpenNetStream.dll` and never hold app credentials.** P3 is
  editor-side only; the proprietary SDK is never redistributed (gitignored, per crate README).
- Playlist stores `stream: { kind: "ezviz_hls", camera: …, lan_fallback: <rtsp-url?> }`;
  the player resolves at runtime: try P2 if configured & reachable, else P1 via broker.

---

## 4. Token Broker Service (backend, new component)

Players must not ship appKey/appSecret. Server-mode token flow per EZVIZ help center:
server obtains `accessToken` + expiry and distributes it; clients call `setAccessToken`. <cite: help/en/491 §Initialization>

```rust
// camera-broker service (part of compile/deploy backend)
- holds appKey/appSecret (per-region: isgpopen intl / areaDomain / ys7 China)
- mints short-lived tokens and distributes them to players
- renews on 10002 (token expired) with one-shot auto-refresh (crate behavior)
- rate-limit aware: caches resolved live URLs per camera (explicit expireTime,
  e.g. 24 h), never per-player fan-out (10007/10029 protection at fleet scale)
- player → broker: mTLS or device token; broker → EZVIZ: app credentials
```

- **C1:** EZVIZ tokens are **per-appKey/region, not per-player** — there is no
  per-player token scoping in the platform. The real scoping mechanism is the
  **live-URL `id`** (revocable via `/api/lapp/live/address/disable`) plus rate
  limits. The broker should mint one token per appKey/region and scope *URLs*
  per camera, not tokens per player.
- **Q2 (deferred to Phase 2):** broker topology. Default to evaluate first: a
  **Rust sidecar** wrapping the tested `ezviz-openapi` crate, exposed as a local
  HTTP service Laravel proxies — reuses the tested token/URL/error logic and
  avoids PHP↔Rust protocol drift. Pure-PHP reimplementation is the fallback if
  the broker must live entirely in Laravel (accept the drift cost explicitly).

Playlist auth block:
```json
"auth": { "kind": "broker_token", "endpoint": "https://broker…/ezviz/token",
          "refresh_secs": 4200 }
```

---

## 5. Playlist-v1 Additions (SPEC-006 delta)

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

Player semantics: alarm event → swap to bound `live_stream` at interrupt priority →
dwell → resume playlist. Camera capability flags (`isSupportTalk`, PTZ support levels)
are admin-only metadata; players ignore them.

**Q3 (decided): alarm delivery.** The **broker polls EZVIZ once per camera**
(`/api/lapp/alarm/list`) and **pushes events to players** over WebSocket/SSE.
Players never poll EZVIZ directly. EZVIZ's MQTT Message System (help/en/2392)
is the scale path to evaluate later; per-player polling is rejected (N pollers
per camera at fleet scale).

---

## 6. Editor / Admin Integration

- **CCTV widget in editor canvas (Phase 2):** live preview via broker (P1 HLS in a
  video element is fine for preview); PTZ panel via `CameraProvider::ptz_*`.
- **Windows admin build only:** P3 low-latency preview via `pc-sdk` feature
  (HWND overlay window). Feature-gated; Linux/macOS admin uses P1.
- **Verify-code / stream encryption UX:** if device has encryption enabled
  (`isEncrypt`), stream needs the 6-char verification code. **C2:** the
  `400035`/`400036` codes cited are **mobile-SDK codes** — the HTTP-path
  verify-code error codes must be confirmed empirically, and encrypted-device
  HLS may require the `code` param in `live/address/get`. On error, prompt and
  retry. <cite: help/en/491 §Live View>
- Video definition (`videoLevel`): set before/during preview; takes effect on
  stop→start cycle (SDK-documented behavior — do not expect hot-swap).

---

## 7. Security & Compliance

1. No appKey/appSecret on any player device; broker-only issuance.
2. Verification codes stored server-side, never in playlists; delivered to players
   over the device-auth channel only when a P2 RTSP path requires them.
3. Proprietary SDK (`EZPCOpenSDK_v5.1.18/`) never committed or redistributed;
   `pc-sdk` loads at runtime via `libloading` (crate invariant).
4. Privacy: camera feeds in public signage need jurisdictional review (retention,
   signage of CCTV operation). Flagged per-deployment, not per-template.
5. REA guardrails apply (research/REA_RESEARCH_PLAN.md §2): REA only if SDK/behavior
   diverges from docs; findings logged in research format.

---

## 8. Error Model (unified)

| Source | Code | Meaning | Platform behavior |
|---|---|---|---|
| HTTP | 10002 | token expired | broker auto-refresh once, retry; else player holds last frame + fallback card |
| HTTP | 10005/10007/10029 | frozen / rate-limited | broker backoff + jitter; player unaffected (cached URL) |
| HTTP | 20002/20007/20018 | device missing/offline/illegal | widget → `on_stream_lost` fallback |
| HTTP | 60000–60011 | PTZ unsupported/limit/fail | admin UI toast; never player-visible |
| SDK (P3) | 46/47/19 | PTZ ok / failed / exception | admin preview message callback (crate `pc_sdk.rs`) |
| Stream | verify-code (HTTP codes TBD — C2) | code required/mismatch | admin re-prompt flow; player → fallback |
| Player | any | stream stall > 5 s | `on_stream_lost` fallback; alarm to fleet monitor |

---

## 9. Acceptance Criteria

- [ ] Player renders HLS camera widget at 1080p alongside playlist video; stream kill
      → fallback card ≤ 5 s; recovery auto within 2 refresh cycles.
- [ ] **C5:** P1 glass-to-glass ≤ 15 s (tuned HLS); LAN path: player on camera VLAN
      plays RTSP (P2) with < 1 s glass-to-glass; cloud fallback engages when RTSP
      probe fails.
- [ ] Alarm interrupt: injected alarm → fullscreen camera ≤ 3 s end-to-end; dwell →
      playlist resumes at correct item boundary.
- [ ] 500 players share one camera: broker resolves 1 upstream URL (cached), ≤ 1
      token refresh/hour/player, zero 10007/10029 errors in soak.
- [ ] No `OpenNetStream.dll`, appKey, or appSecret present in any player artifact
      (release lint: REA Rust-binary scan + string grep in CI — research plan O1/O6).
- [ ] PTZ control from Windows admin build verified against live camera; Linux/macOS
      admin degrades to P1 preview with PTZ via HTTP.

---

## 10. Phasing & Traceability

| Phase | Work | Gates |
|---|---|---|
| 2 (editor) | CCTV widget, broker MVP (**Q2 topology decided here**), admin PTZ panel, verify-code UX | AC §9 (admin-side) |
| 3 (player) | GStreamer HLS/RTSP paths (**Q1 texture-upload spike first**), interrupt engine, alarm poller (**Q3 broker-push**), fallback matrix, **Q4 LAN discovery decided here** | AC §9 (player-side) + G3 48 h soak with simulated alarms |
| 4 | Multi-provider (Hikvision ISAPI/ONVIF) behind `CameraProvider`; fleet camera analytics | G4 |

Traceability: extends SPEC-001 (WidgetInstance), SPEC-003 (DataBroker ← alarm/URL
sources), SPEC-005 (`BindAlarmTrigger`, `PlayerControl`), SPEC-006 (playlist items
above), SPEC-007 (fallback/interrupt/watchdog interplay), SPEC-008 (snapshot assets
via content-hash store). Research log: `research/` items S1–S5 inform §3/§5/§8.

---

## 11. Decision Log (grill-me, 2026-10-17)

| ID | Decision | Verdict |
| --- | --- | --- |
| Q1 | Decode → compositor path | **GStreamer decode → texture upload (D3D11/DMA-BUF) → wgpu texture.** Overlay windows / libmpv are fallbacks only on evidence. |
| Q2 | Broker topology | **Deferred to Phase 2.** Default to evaluate: Rust sidecar wrapping `ezviz-openapi`, Laravel proxies. |
| Q3 | Alarm delivery | **Broker polls EZVIZ once per camera + pushes to players (WebSocket/SSE).** MQTT Message System = later scale path. |
| Q4 | Camera LAN discovery (P2) | **Deferred to Phase 3.** Default: manual admin entry; mDNS/UPnP later. |
| Q5 | Snapshot trait shape | **`snapshot_url()` returns a URL, caller fetches.** Never `Bytes` synchronously. |
| C1 | Token scoping | Corrected: tokens are per-appKey/region, not per-player. Scoping = live-URL `id` (revocable) + rate limits. |
| C2 | Verify-code codes | Corrected: 400035/400036 are mobile-SDK codes; HTTP-path codes + `code` param for encrypted HLS must be confirmed empirically. |
| C3 | Snapshot semantics | Corrected: `capture` is fire-and-forget; picture arrives async. |
| C4 | Alarm scaling | Corrected: per-player polling rejected; broker-push (Q3). |
| C5 | HLS latency | Corrected: 10–30 s is untuned worst case; tune `gst-hls` to 5–15 s; P1 AC added. |

# CCTV integration — signage platform

How a signage platform consumes this crate: a live-camera widget, an
alarm-driven interrupt, and a best-effort snapshot. Everything here is
**consumer-side**; the crate itself is documented in
[`01-api-reference.md`](01-api-reference.md).

> The platform specs this extends (widgets, DataBroker, `BindAlarmTrigger`,
> playlist, fallback/interrupt, assets) live in the signage repository. They are
> referenced by name and intentionally unresolved from here.

## 1. Shape of the integration

Three capabilities, one provider boundary — not a separate application:

| Capability | Behaviour | Crate calls |
|---|---|---|
| Live widget | camera renders inside a template | `live_hls_address()`, optionally `disable_live_address()` |
| Alarm interrupt | alarm → full-screen camera → dwell → resume playlist | `list_alarms()` (broker-side polling) |
| Snapshot widget | periodic image refresh | `capture()` + fetch of the platform picture URL |

Out of scope: two-way audio, Wi-Fi provisioning, cloud recording download, device
add/activation (done in the EZVIZ app).

## 2. Provider boundary

The platform never handles crate types above this trait:

```rust
// platform-side; EzvizProvider adapts EzvizClient to it
#[async_trait]
pub trait CameraProvider: Send + Sync {
    async fn live_url(&self, cam: &CameraRef, proto: StreamProtocol,
                      quality: VideoQuality) -> Result<StreamUrl, CameraError>;
    async fn snapshot_url(&self, cam: &CameraRef) -> Result<Option<String>, CameraError>;
    async fn poll_alarms(&self, since: SystemTime) -> Result<Vec<Alarm>, CameraError>;
    async fn ptz_start(&self, cam: &CameraRef, dir: PtzDirection, speed: PtzSpeed) -> Result<(), CameraError>;
    async fn ptz_stop(&self, cam: &CameraRef, dir: PtzDirection) -> Result<(), CameraError>;
}
```

Adapter mapping — the only place crate types appear:

| Trait | `EzvizClient` | Rules |
|---|---|---|
| `live_url` | `live_address(…)` / `live_hls_address(…)` | HLS is the portable default |
| `snapshot_url` | `capture(serial, ch)` | returns `Result<()>`; the URL comes from the alarm/picture field, not from `capture` |
| `poll_alarms` | `list_alarms(ms_start, ms_end, type, status, page, size)` | clamp `since` to 7 days, page to the end |
| `ptz_start` | `ptz_start(serial, ch, dir, speed)` | explicit stop required |
| `ptz_stop` | `ptz_stop(serial, ch, dir)` | same direction |

Types: `CameraRef` → `(device_serial, channel_no)`; `Alarm` → `AlarmItem`;
`CameraError` → `EzvizError`.

## 3. Streaming tiers

| Tier | Path | Latency | Where |
|---|---|---|---|
| **P1 cloud** | `live/address/get` → HLS → GStreamer `souphttpsrc ! hlsdemux` | 5–15 s tuned, 10–30 s untuned | player runtime, default |
| **P2 LAN** | `rtsp://<cam-ip>:554/h264/ch1/<ch>/av_stream` | < 1 s | player on the camera VLAN |
| **P3 admin** | PCOpenSDK `OpenSDK_StartRealPlayEx` into an HWND (`pc-sdk`) | < 1 s | Windows editor only |

**P2 is not minted by this crate.** The API has no RTSP protocol value
(`Ezopen/Hls/Rtmp/Flv` only), so the RTSP URL is constructed platform-side from
a LAN address the admin enters, and the verification code is supplied
separately. Treat it as an optional fast path that probes, not as a guaranteed
capability — many models ship with RTSP disabled.

**P3 conflicts with a compositor.** HWND embedding cannot feed a wgpu texture,
so it is admin preview only; players never load the DLL and never hold app
credentials.

**Decoder path.** Frames reach the compositor as
GStreamer decode → texture upload (D3D11 / DMA-BUF) → wgpu texture, so cameras
z-order and transform like any other widget. Overlay windows and libmpv are
fallbacks only if texture sharing proves unworkable.

## 4. Token broker

Players must not ship `appKey`/`appSecret`.

```
player ──mTLS/device token──▶ broker ──appKey/appSecret──▶ EZVIZ
```

- The broker holds the credential pair per region and mints tokens; it is also
  the only component allowed to call `live/address/get` at fleet scale.
- **Tokens are per appKey/region, not per player.** Scoping is per *URL*: the
  `id` from `LiveAddress` is revocable with `disable_live_address()`. Rate
  limits (`10007`/`10029`) are the real constraint; per-player tokens do not
  help.
- Cache resolved URLs per camera with their platform expiry; never fan out one
  upstream request per player.
- Topology (Phase 2): a Rust sidecar wrapping this crate, proxied by Laravel.
  A pure-PHP reimplementation is the fallback and pays an explicit drift cost.

Playlist auth block:

```json
"auth": { "kind": "broker_token", "endpoint": "https://broker…/ezviz/token", "refresh_secs": 4200 }
```

## 5. Playlist items

```jsonc
// live camera
{ "type": "live_stream",
  "camera": { "provider": "ezviz", "device_serial": "…", "channel_no": 1 },
  "protocol_pref": ["rtsp_lan", "hls"],
  "quality": "fluent",
  "auth": { "kind": "broker_token", "refresh_secs": 4200 },
  "rect": [100, 200, 800, 450],
  "on_stream_lost": { "action": "fallback_asset", "asset": "offline_card.png" },
  "interruptible": true }

// snapshot (best-effort)
{ "type": "camera_snapshot", "camera": { "…": "…" }, "refresh_secs": 60, "on_failure": "hide" }

// alarm interrupt binding
{ "type": "alarm_trigger", "camera": { "…": "…" }, "alarm_types": ["motion"],
  "action": { "type": "live_stream_fullscreen", "dwell_secs": 30, "then": "resume" } }
```

**Alarm delivery.** The broker polls `alarm/list` once per camera and pushes to
players over WebSocket/SSE. Per-player polling is rejected: N players × M
cameras is not a load the platform tolerates. EZVIZ's MQTT message service is
the later scale path.

## 6. Editor / admin

- CCTV widget in the canvas with an HLS preview (a plain `<video>` is adequate
  for preview) and a PTZ panel over `ptz_*`.
- Windows builds may use P3 for a low-latency preview; Linux/macOS use P1.
- **Encrypted devices** (`isEncrypt`) need the 6-character verify code. The
  `400035`/`400036` codes are mobile-SDK codes; the HTTP equivalents and whether
  encrypted HLS requires the `code` parameter are unconfirmed — prompt and
  retry on any verify-code error rather than branching on a guessed number.
- **`videoLevel`** takes effect on a stop→start cycle; do not expect a hot swap.

## 7. Security and compliance

1. No `appKey`/`appSecret` on any player artifact.
2. Verify codes are server-side; sent to a player only when its LAN RTSP path
   needs one.
3. The proprietary SDK is never committed or redistributed; `pc-sdk` loads it
   at runtime. `EZPCOpenSDK_v5.1.18/` is gitignored.
4. Camera feeds in public signage need jurisdictional review (retention, CCTV
   signage) per deployment, not per template.
5. Reverse engineering is a last resort, used only where documented behaviour
   diverges from observed behaviour — see
   [`03-external-refs.md`](03-external-refs.md).

## 8. Error handling

| Source | Signal | Platform reaction |
|---|---|---|
| API | `10002` | crate refreshes once; if it repeats, broker re-mints |
| API | rate limit (`10007`/`10029` per vendor docs) | broker backs off with jitter; players keep the cached URL |
| API | device missing/offline | widget shows `on_stream_lost` fallback |
| API | PTZ failure | admin toast only; never player-visible |
| SDK | `47` / `19` | admin preview message |
| Stream | stall > 5 s | fallback card + fleet monitor alarm |

The crate preserves the platform's `code` and `msg` verbatim in
`EzvizError::Api` — log them; the numeric ranges differ between regions.

## 9. Acceptance criteria

- [ ] HLS widget renders at 1080p beside playlist video; killing the stream
      shows the fallback card within 5 s and recovers within two refresh cycles.
- [ ] P1 glass-to-glass ≤ 15 s tuned; P2 on the camera VLAN < 1 s; cloud
      fallback engages when the RTSP probe fails.
- [ ] Alarm → full-screen ≤ 3 s end-to-end; dwell then correct playlist resume.
- [ ] 500 players, one camera: one upstream URL resolved, ≤ 1 token refresh per
      hour, zero rate-limit errors in soak.
- [ ] No DLL, `appKey` or `appSecret` in any player artifact.
- [ ] PTZ verified on Windows admin against a live camera; Linux/macOS admin
      degrades to P1 preview with HTTP PTZ.

## 10. Phasing

| Phase | Work | Gate |
|---|---|---|
| 2 (editor) | CCTV widget, broker MVP, admin PTZ panel, verify-code UX | admin AC above |
| 3 (player) | HLS/RTSP paths (texture-upload spike first), interrupt engine, broker-push alarms, fallback matrix | player AC + 48 h soak |
| 4 | second provider (ISAPI/ONVIF) behind `CameraProvider` | — |

## 11. Decisions

| ID | Topic | Verdict |
|---|---|---|
| Q1 | Decode path | GStreamer → texture upload → wgpu texture. Overlays/libmpv only on evidence. |
| Q2 | Broker topology | Deferred to Phase 2. Default: Rust sidecar, Laravel proxies. |
| Q3 | Alarm delivery | Broker polls per camera, pushes to players. MQTT later. |
| Q4 | LAN discovery | Deferred to Phase 3. Manual admin entry now. |
| Q5 | Snapshot shape | `snapshot_url()` returns a URL; the caller fetches. Never `Bytes`. |
| C1 | Token scoping | Per appKey/region, not per player; scope URLs, not tokens. |
| C2 | Verify-code codes | `400035`/`400036` are mobile-SDK codes; HTTP codes unconfirmed. |
| C3 | Snapshot semantics | `capture` is fire-and-forget. |
| C4 | Alarm scaling | Per-player polling rejected in favour of broker push. |
| C5 | HLS latency | 10–30 s untuned; tune `gst-hls` to 5–15 s. |

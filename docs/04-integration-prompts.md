# Integration prompts

Copy-paste prompts for embedding `ezviz-openapi` in a host application. Each is
self-contained: exact APIs, the platform rules that are easy to get wrong, and
what "done" means. Paste them into an agent (or a colleague) working **inside
the host repository** — this crate is the dependency, not where the feature
lives.

```toml
[dependencies]
ezviz-openapi = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Five rules every scenario inherits:

1. **Credentials are server-side.** `appKey`/`appSecret` never ship to a player,
   a browser or a published binary. A broker mints tokens; consumers receive
   short-lived material.
2. **Tokens are per appKey per region, not per player, and not per device.**
   Scope *URLs* instead: every `LiveAddress` has an `id` revocable with
   `disable_live_address()`.
3. **`areaDomain` wins over your configured host.** The client switches its base
   URL to the region the token was issued for; a "valid" token against the wrong
   host fails.
4. **The crate returns URLs; it does not decode video.** Rendering, retry and UI
   belong to the host.
5. **No `unwrap()` on the client.** Branch on `EzvizError::{Api, Http, Json, Io,
   Sdk}` — `Api` preserves the platform's own `code`/`msg`, which is the only
   reliable way to explain a failure to an operator.

---

## 1. Live camera widget (HLS)

**Prompt**

> Add a live EZVIZ camera view using `ezviz-openapi`.
>
> - Resolve the URL with `client.live_hls_address(device_serial, channel_no,
>   VideoQuality::Fluent)` for players (HD for admin previews). HLS is the
>   portable choice; `StreamProtocol::Ezopen` is unplayable outside the vendor
>   SDK and RTMP/FLV need their own demuxer.
> - Cache the URL until its `expire_time`; do not re-resolve per view. Resolve
>   once per camera in the backend and hand players a URL (or a proxied stream),
>   never the credential.
> - Handle expiry by re-resolving and swapping the source, keeping the last frame
>   visible meanwhile. A 403/stall means resolve again, not "camera offline".
> - Call `disable_live_address(serial, channel, &id)` when a stream is torn
>   down, so the platform can reclaim the slot.
>
> Done when: the widget plays from a cold start, survives a URL expiry without a
> black flash, and a torn-down widget releases its address.
>
> Out of scope: transcoding, seeking, audio.

## 2. Token broker (backend service)

**Prompt**

> Build the credential-holding broker for EZVIZ using `ezviz-openapi`, exposed
> as an internal HTTP endpoint.
>
> - The broker holds `appKey`/`appSecret`, constructs
>   `EzvizClient::new(key, secret)` and mints tokens by calling
>   `access_token()`. It is the **only** process that sees the credential pair.
> - Cache one token per region (`areaDomain`) and hand it out with its expiry —
>   tokens are per appKey, so minting one per request multiplies rate limiting.
> - Serve resolved live URLs per camera from the same cache (keyed by
>   `deviceSerial/channelNo`) to avoid one upstream call per player.
> - Protect the endpoint with mTLS or a device token. It returns URLs and
>   tokens; it never returns appKey/appSecret.
> - On `EzvizError::Api` with code `10002`, refresh once and retry; the crate
>   already retries inside a single call, so a second `10002` means the
>   credential is wrong, not the token.
>
> Done when: 500 clients on one camera cause exactly one upstream URL resolution
> per expiry window, and rotating the app secret needs no client change.
>
> Out of scope: per-player tokens (they do not exist), user login flows.

## 3. PTZ admin panel

**Prompt**

> Add PTZ control to the admin UI using `ezviz-openapi`.
>
> - `ptz_start(serial, channel, PtzDirection, PtzSpeed)` starts a continuous
>   movement; it **must** be paired with `ptz_stop(serial, channel, same
>   direction)`. There is no server-side auto-stop, so a lost "pointer up" event
>   leaves the camera moving — add a client-side watchdog that stops after N
>   seconds without input.
> - Directions: `Up/Down/Left/Right`, four diagonals, `ZoomIn/ZoomOut`,
>   `FocusNear/FocusFar`. Speeds: `Slow/Medium/Fast`.
> - Expose `ptz_mirror` and presets (`preset_add` returns the index you later
>   pass to `preset_move` / `preset_clear`).
> - PTZ is admin-only. A player must never expose it, and PTZ failures are toasts
>   in the editor, never player-visible errors.
>
> Done when: a drag-less button press moves and reliably stops the camera, and a
> browser tab closed mid-move does not leave it panning.
>
> Out of scope: joystick gesture design, video rendering.

## 4. Alarm poller → push notifications

**Prompt**

> Build an alarm ingestion service with `ezviz-openapi`.
>
> - Poll `list_alarms(start_ms, end_ms, AlarmType::All, AlarmStatus::Unread,
>   page_start, page_size)` **once per camera** in the broker, and push events to
>   clients over SSE/WebSocket. Per-client polling is rejected: N clients × M
>   cameras is not a load the platform tolerates.
> - Timestamps are Unix **milliseconds** and the window is capped at **7 days**.
>   Keep a durable cursor per camera, clamp a cold start to 7 days, and page
>   until a short page comes back.
> - Handle `AlarmType::All` as the enum value `-1`, not an empty filter, and mark
>   events read with the status filter only when the product needs it.
> - Emit the platform's `alarmPicUrl` with the event; the crate does not download
>   images.
>
> Done when: a restart resumes from the cursor without gaps or duplicates, and a
> 10-minute outage produces one catch-up sweep rather than a flood.
>
> Out of scope: MQTT push (a later scale path), image storage.

## 5. Snapshot widget (periodic JPEG)

**Prompt**

> Add a periodic snapshot tile using `ezviz-openapi`.
>
> - `capture(serial, channel).await?` **only requests** the picture — it returns
>   `Result<()>` and no URL. Fetch the image afterwards from the platform picture
>   URL (the same field alarms carry), with a short delay to let the device
>   produce it.
> - Treat the whole path as best-effort: on any failure keep the previous image
>   and mark the tile stale; never blank the tile or retry in a tight loop.
> - Refresh at a human cadence (30–120 s) and stagger cameras so a wall of tiles
>   does not burst the API.
>
> Done when: a tile refreshes on schedule, a camera offline for hours shows a
> stale marker rather than an error dialog, and no request storm appears in the
> broker log.
>
> Out of scope: image resizing/caching infrastructure (use the host's).

## 6. NVR / multi-channel grid

**Prompt**

> Render a grid of channels from an NVR using `ezviz-openapi`.
>
> - Enumerate with `device_cameras(serial)` (or `list_cameras()` for the whole
>   account); each `CameraChannel` carries `channel_no`, name and `is_encrypt`.
> - Every stream request takes `(deviceSerial, channelNo)` — a wrong pair returns
>   an API error, not an empty stream, so validate channel numbers against the
>   enumeration instead of guessing `1..n`.
> - Encrypted channels need the verify code; prompt for it once per device and
>   pass it as `safe_code` on playback/stream requests.
> - Resolve URLs through the broker and reuse HLS for all tiles; mixing protocols
>   across a grid multiplies decoder support requirements.
>
> Done when: an 8-channel grid opens in one pass, one offline channel does not
> block the others, and encrypted channels prompt exactly once.
>
> Out of scope: per-tile PTZ, tour/patrol automation.

## 7. Playback scrubber (local or cloud recordings)

**Prompt**

> Add recording playback with `ezviz-openapi`.
>
> - Use `local_playback_address(serial, ch, quality, start, stop)` for SD-card
>   recordings and `cloud_playback_address(…)` for CloudPlay. Times are
>   `yyyy-MM-dd HH:mm:ss`.
> - A new window is a **new URL**: resolve on every seek boundary, don't try to
>   seek inside an HLS URL the platform minted for a fixed window.
> - `expireTime` is ignored for playback; the window (`startTime`/`stopTime`) is
>   what bounds it.
> - Release old addresses with `disable_live_address` when a window is replaced.
>
> Done when: seeking backwards and forwards beyond the current window works by
> re-resolving, and no more than one address per window is live at a time.
>
> Out of scope: frame-accurate scrubbing, thumbnail strips.

## 8. Windows desktop P2P preview (`pc-sdk`)

**Prompt**

> Add low-latency preview to the Windows desktop build using the `pc-sdk`
> feature.
>
> - `PcSdk::load()` resolves `OpenNetStream.dll` from `$EZVIZ_SDK_DIR`, the
>   working directory, then `PATH`. Ship those DLLs beside the app; the crate
>   never redistributes them.
> - Lifecycle: `init(app_id)` → `set_access_token(token)` →
>   `alloc_session_ex(None, hwnd)` → `start_real_play_ex(&session, hwnd, serial,
>   channel, safe_key)` → `stop_real_play_ex` → `free_session` → `fini_lib`. One
>   session per window; never leak a session on panic — wrap in a guard.
> - The rendered surface is a window handle: it cannot feed a GPU texture, so
>   this is an editor/admin preview, never the player compositor.
> - Gate the whole module behind `#[cfg(feature = "pc-sdk")]` and keep the HTTP
>   client as the fallback path on every other platform.
>
> Done when: preview starts under 1 s on Windows, closing the window frees the
> session, and the Linux/macOS build still compiles without the feature.
>
> Out of scope: container packaging of the DLLs, macOS support.

## 9. Test the integration without credentials

**Prompt**

> Add tests for the EZVIZ integration that need no account and no camera.
>
> - Use `EzvizClient::with_token(token, area_domain)` to skip minting, and
>   `set_base_url()` to point the client at a local mock (wiremock) — the public
>   seam that makes the crate testable.
> - Assert on the real envelope shapes: `{code, msg, data, page}` with `code`
>   as a string **and** as an integer (regions differ), a `data`-less ack
>   response, and a `10002` that must trigger exactly one retry.
> - Cover the failure branches that matter: `EzvizError::Api` carrying the
>   platform's message, and a malformed `data` surfacing as `EzvizError::Json`.
> - Never assert on live device names, serials or timings.
>
> Done when: the full integration path is exercised in CI in under a second, and
> the only tests needing hardware are explicitly ignored.

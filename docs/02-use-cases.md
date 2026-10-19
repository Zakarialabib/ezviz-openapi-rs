# Use cases

Six ways this crate gets assembled into a product. Each one states the shape, the
crate calls involved, and the rule that decides whether you are doing it right.
Pick the one that matches yours and ignore the rest.

A digital-signage player, a shop's camera wall and an NVR dashboard are the same
three problems in different clothes: get a URL, keep it fresh, and never put
credentials where a viewer can reach them.

## 1. Live view (HLS)

**Shape.** Backend resolves a URL per camera; the frontend plays it.

```rust
let live = client
    .live_hls_address(serial, channel, VideoQuality::Fluent)
    .await?;
// hand `live.url` to the player; keep `live.id` to revoke it later
```

- Players get **Fluent**; admin previews get **Hd**.
- Cache until `expire_time`. An expired URL is not an offline camera — re-resolve
  before showing an error.
- Call `disable_live_address(serial, channel, &id)` when a stream is torn down.

**Rule.** One URL per camera per expiry window, never one per viewer.

## 2. Token broker

**Shape.** A backend service holds the app key/secret and hands out tokens and
URLs.

```rust
let broker = EzvizClient::new(app_key, app_secret);   // only this process
let token = broker.access_token().await?;
// consumers use: EzvizClient::with_token(token, area_domain)
```

- The broker is the only process with the credential pair; it never returns it.
- One token per **region**, cached, reused — not one per request.
- Cache resolved URLs per camera too; that is what keeps a fleet off the rate
  limits.
- Protect the endpoint with mTLS or a device token.

**Rule.** Credentials stop at the broker; consumers receive short-lived material.

## 3. Alarm pipeline

**Shape.** One poller per camera in the broker, events pushed onward.

```rust
let page = client
    .list_alarms(start_ms, end_ms, AlarmType::All, AlarmStatus::Unread, 0, 50)
    .await?;
```

- Timestamps are **Unix milliseconds**; the window is capped at **7 days**.
- Persist a cursor per camera and page until a short page returns. A restart must
  resume, not re-scan.
- Emit `alarmPicUrl` with the event; the crate does not download images.
- Push to consumers over SSE/WebSocket. Per-consumer polling does not scale.

**Rule.** Poll once per camera, fan out from there.

## 4. Snapshots

**Shape.** Periodic refresh of a still image.

```rust
client.capture(serial, channel).await?;   // request only — no URL comes back
```

- `capture()` is fire-and-forget: fetch the picture from the platform picture URL
  afterwards, with a short delay.
- Failure keeps the previous image and marks it stale; a tile must never go blank
  or retry in a tight loop.
- Refresh at a human cadence (30–120 s), staggered across cameras.

**Rule.** Best-effort by design; treat it as an enrichment, not a source of truth.

## 5. Playback with a scrubber

**Shape.** Resolve a window, play it, re-resolve when the window changes.

```rust
let clip = client
    .cloud_playback_address(serial, channel, VideoQuality::Fluent, "2026-10-01 08:00:00", "2026-10-01 09:00:00")
    .await?;
```

- A window is a **new URL**: seeking outside it re-resolves rather than seeking
  inside an HLS URL minted for a fixed range.
- Use `local_playback_address` for SD-card recordings, `cloud_playback_address`
  for cloud recordings.
- `expireTime` is ignored for playback; the window bounds it.

**Rule.** One live address per window; release the old one when you swap.

## 6. Multi-channel recorder (NVR)

**Shape.** A grid over one device's channels.

```rust
for channel in client.device_cameras(serial).await? {
    // channel.channel_no, channel.name, channel.is_encrypt
}
```

- Take channel numbers from the enumeration; `(deviceSerial, channelNo)` is a
  pair, and a wrong pair is an API error rather than an empty stream.
- Encrypted channels need the verify code once per device, passed as `safe_code`.
- Resolve all tiles through the broker with the same protocol (HLS) — a mixed grid
  multiplies decoder requirements.

**Rule.** Enumerate first, then request; never guess channel indexes.

## 7. Windows desktop preview

**Shape.** Native P2P rendering inside a desktop app (`pc-sdk`).

```rust
let sdk = PcSdk::load()?;
sdk.init(app_id)?;
sdk.set_access_token(&token)?;
let session = sdk.alloc_session_ex(None, hwnd)?;
sdk.start_real_play_ex(&session, hwnd, serial, channel, None)?;
// stop_real_play_ex → free_session → fini_lib
```

- The DLL loads from `$EZVIZ_SDK_DIR` / cwd / `PATH`; nothing is bundled.
- Render target is a window handle, so this cannot feed a GPU texture pipeline —
  it is an editor/preview surface, not a compositor input.
- Wrap the session in a guard: a panic must not leak a session.

**Rule.** Native preview is an accelerator for one screen, never the only path.

## Cross-cutting rules

| Rule | Why |
|---|---|
| Credentials server-side only | a player with an app key is a leaked app key |
| Tokens per region, URLs per camera | that is the platform's scoping model |
| Handle `EzvizError::Api` by code/msg | the numeric ranges differ by region; the message is the honest signal |
| Never `unwrap()` a client call | a camera going offline is normal, not exceptional |
| Re-resolve instead of retrying a failing URL | URLs expire; retrying an expired one always fails |

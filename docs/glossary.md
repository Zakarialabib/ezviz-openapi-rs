# Glossary

Durable definitions for the EZVIZ Open Platform surface this crate wraps. Terms
are the ones the API, the SDK and the signage consumer actually use — not
paraphrases.

## Credentials

- **appKey / appSecret** — the per-application credential pair issued by the
  EZVIZ Open Platform console. Belong to the *server*, never to a player or a
  published binary.
- **accessToken** — short-lived bearer token minted by
  `POST /api/lapp/token/get` from appKey/appSecret. This crate caches it in
  memory and refreshes once when a call returns `10002`.
- **areaDomain** — the regional API host a token is bound to. Tokens are
  per-appKey **per region**, not per device or per user: using a token against
  the wrong host fails even though the credential is valid.
- **verify code** (also "safe code", `code`) — the 6-character device
  encryption code. Required for encrypted devices on playback/stream URLs. Not
  an account credential and not interchangeable with appSecret.

## Devices and channels

- **deviceSerial** — the device identity used by every endpoint
  (`deviceSerial` parameter). Not the same string as a MAC address.
- **channelNo** — the channel index on a device. A single-lens camera is
  channel `1`; NVR channels are `1..n`. Every call that acts on a camera takes
  both `deviceSerial` and `channelNo`.
- **CameraChannel / DeviceInfo / DeviceDetail** — the crate's typed views of
  `/api/lapp/camera/list`, `/api/lapp/device/info` and
  `/api/lapp/device/list` rows respectively.

## Streaming

- **live address** — a play URL minted by `POST /api/lapp/live/address/get`,
  identified by an `id` that can be revoked with
  `/api/lapp/live/address/disable`. URLs carry an expiry.
- **ezopen** — EZVIZ's own playback protocol/URL scheme (`protocol=1`). Works
  only with the vendor SDK/player; this crate can request it but cannot play
  it.
- **HLS / FLV / RTMP** — `protocol=2/4/3`. HLS (`.m3u8`) is the portable
  choice for a signage player; H264-over-FLV is lower latency but needs an FLV
  demuxer.
- **LiveType** — preview (`1`) versus local SD-card playback (`2`) or cloud
  playback (`3`). Playback types require `startTime`/`stopTime` as
  `yyyy-MM-dd HH:mm:ss`.
- **VideoQuality** — `Hd` (main bitrate, `1`) or `Fluent` (sub bitrate, `2`).

## Control

- **PTZ** — pan/tilt/zoom. `PtzDirection` covers the 8 directions plus zoom and
  focus (`0..11`); a movement must be followed by an explicit `ptz_stop` with
  the same direction.
- **mirror** — horizontal/vertical flip of the image, a display transform
  rather than a PTZ move.
- **preset** — a stored camera position that can be added, moved to, or
  cleared.
- **capture** — a device-side snapshot request. It is fire-and-forget: the
  call only asks for the picture, and the image is fetched later from the
  platform's picture URL.

## Alarms

- **AlarmItem** — one row from `/api/lapp/alarm/list`. Carries the picture URL,
  type and time.
- **AlarmType / AlarmStatus** — typed filters. The query window is Unix
  **milliseconds** and is capped at **7 days**; results are paginated.

## SDK / integration terms

- **PCOpenSDK (`OpenNetStream.dll`)** — the proprietary Windows native SDK the
  optional `pc-sdk` feature binds at runtime. Never bundled or redistributed by
  this crate.
- **HWND preview** — SDK real-play renders into a window handle provided by the
  caller (`OpenSDK_StartRealPlayEx`). Incompatible with a GPU compositor that
  expects textures.
- **Gap (A–E)** — the reverse-engineering gaps from the signage playbook
  (`docs/01-cctv-integration.md` §11): certificate pinning, token math,
  undocumented payloads, AES-GCM framing, protobuf schemas. Unimplemented here
  by design.
- **REA** — reverse-engineering tooling (JADX/Apktool driven) used only when
  documented behaviour diverges from what the apps actually send.

# Glossary

The vocabulary of the EZVIZ Open Platform as this crate uses it. These are the
terms integrators get wrong, not a restatement of the README.

## Credentials and region

**appKey / appSecret** — per-application credential pair from the EZVIZ console.
Server-side only: a player, a published binary or a playlist must never carry
them.

**accessToken** — short-lived bearer minted by `POST /api/lapp/token/get` from
the appKey/appSecret pair. Sent as a form field on every authenticated call.

**areaDomain** — the region-scoped API host a token is valid against. Returned in
the token response; the client switches its base URL to it automatically. A
valid token against the wrong host fails — that is a region mismatch, not a bad
credential.

**verify code** (`code`, "safe code") — 6-character device encryption code,
required for encrypted devices on playback/stream requests. Not an account
credential. The vendor's `400035`/`400036` error numbers are mobile-SDK codes;
their HTTP equivalents are unconfirmed in this crate.

**token scoping** — tokens are per appKey **per region**, not per device or per
caller. Scoping is achieved on *URLs* (each `LiveAddress.id` is revocable), not
by minting more tokens.

## Devices

**deviceSerial** — the device identity used by every endpoint. A different
string from the MAC address; do not substitute one for the other.

**channelNo** — channel index on a device: `1` for a single-lens camera,
`1..n` for an NVR. Nearly every camera call takes `(deviceSerial, channelNo)`
together, and passing the wrong pair returns an API error rather than an empty
result.

**DeviceInfo / DeviceDetail / CameraChannel** — the crate's typed views of
`/device/list`, `/device/info`+`/device/status`, and `/camera/list` rows.
All optional fields are `Option<…>` because regions and firmware return
different subsets.

**isEncrypt** — device/channel flag indicating the stream is encrypted; a verify
code is then required.

**videoLevel** — stream quality setting that takes effect on a stop→start
cycle, not hot.

## Streaming

**live address** — a play URL minted by `POST /api/lapp/live/address/get`,
carrying `id`, `url` and an expiry. Revoke with
`POST /api/lapp/live/address/disable`.

**ezopen** — EZVIZ's own protocol value (`1`) and URL scheme. The crate can
request it but cannot play it; only vendor SDKs/players can.

**HLS / RTMP / FLV** — protocol values `2`, `3`, `4`. HLS (`.m3u8`) is the
portable choice; FLV is lower latency but needs an FLV demuxer.

**LiveType** — `Preview` (`1`), `LocalPlayback` (`2`, SD-card) or
`CloudPlayback` (`3`). Playback types require a `yyyy-MM-dd HH:mm:ss` window.

**VideoQuality** — `Hd` (`1`, main bitrate) or `Fluent` (`2`, sub bitrate).
Fluent is the right default for a fleet of players.

**RTSP** — not an API protocol value. A LAN RTSP URL is built by the caller from
a device address, and not every model enables it. Treat it as an optional fast
path that probes.

## Control

**PTZ** — pan/tilt/zoom. Directions `0..11` cover eight moves plus zoom
(`8`,`9`) and focus (`10`,`11`). A movement continues until an explicit stop
with the same direction; there is no server-side auto-stop.

**mirror** — image flip (horizontal/vertical), a display transform, not a PTZ
move.

**preset** — stored position that can be added (returns its index), recalled, or
cleared.

**capture** — a device-side snapshot *request*. Fire-and-forget: it returns no
URL; the image is fetched later from the platform's picture URL.

## Alarms

**AlarmItem** — one `/api/lapp/alarm/list` row: picture URL, type, time, channel.

**alarm window** — `startTime`/`endTime` are Unix **milliseconds** and the
maximum span is **7 days**, so a long outage needs a cursor plus paging rather
than one wide query.

**AlarmType / AlarmStatus** — typed filters. `AlarmType::All = -1`; status is
`Unread`, `Read` or `All`.

## SDK and process

**PCOpenSDK / OpenNetStream.dll** — proprietary Windows native SDK bound by the
optional `pc-sdk` feature through `libloading`. Never bundled or redistributed;
loaded at runtime from `EZVIZ_SDK_DIR`, the working directory or `PATH`.

**HWND preview** — SDK real-play renders into a caller-supplied window handle.
It cannot feed a GPU texture, so it is an admin/editor surface, not a player one.

**REA** — reverse-engineering tooling used only where documentation and observed
behaviour disagree; see [`03-external-refs.md`](03-external-refs.md).

**consumer / broker** — the platform split: the *broker* holds credentials,
mints tokens and resolves URLs; *consumers* (players, editors) only receive
short-lived material.

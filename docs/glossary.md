# Glossary

The map. Every term this crate or the platform uses, and where to read more. Terms
link to the page that explains them; the source path is where the behaviour is
implemented.

## Credentials and region

**appKey / appSecret** — per-application credential pair from the EZVIZ console.
Server-side only; never in a player, a browser or a published binary. →
[use case 2](02-use-cases.md#2-token-broker), `src/client.rs`.

**accessToken** — short-lived bearer minted by `POST /api/lapp/token/get`. Sent as
a form field on authenticated calls. → [`01-api-reference.md`](01-api-reference.md#token-lifecycle).

**areaDomain** — the region-scoped API host a token is valid against, returned in
the token response. The client switches its base URL to it. A valid token against
the wrong host fails: that is a region mismatch, not a bad credential. →
`src/token.rs`, `src/client.rs`.

**verify code** (`code`, "safe code") — 6-character device encryption code needed
for encrypted devices on stream/playback requests. Not an account credential. The
`400035`/`400036` numbers are mobile-SDK codes; their HTTP equivalents are
unconfirmed (see README limitations).

**token scoping** — tokens are per appKey **per region**, not per device and not
per caller. Scoping is done on URLs (`LiveAddress.id` is revocable), never by
minting more tokens. → [use case 2](02-use-cases.md#2-token-broker).

## Devices

**deviceSerial** — the device identity every endpoint uses. Not the MAC address.

**channelNo** — channel index: `1` on a single-lens camera, `1..n` on a recorder.
Calls take `(deviceSerial, channelNo)` as a pair; a wrong pair is an API error, not
an empty result. → [use case 6](02-use-cases.md#6-multi-channel-recorder-nvr).

**DeviceInfo / DeviceDetail / CameraChannel** — typed views of `/device/list`,
`/device/info`+`/device/status`, and `/camera/list`. Optional fields are
`Option<…>` because regions and firmware differ. → `src/device.rs`.

**isEncrypt** — device/channel flag: the stream is encrypted, a verify code is
required.

**videoLevel** — stream quality setting that applies on a stop→start cycle, not
hot.

## Streaming

**live address** — a play URL from `POST /api/lapp/live/address/get`, carrying
`id`, `url`, expiry. Revoke with `/api/lapp/live/address/disable`. →
`src/live.rs`.

**ezopen** — EZVIZ's own protocol value (`1`) and URL scheme. Requestable here,
playable only in vendor SDKs.

**HLS / RTMP / FLV** — protocol values `2`, `3`, `4`. HLS (`.m3u8`) is the portable
choice; FLV is lower latency but needs an FLV demuxer. → [use case 1](02-use-cases.md#1-live-view-hls).

**LiveType** — `Preview` (`1`), `LocalPlayback` (`2`), `CloudPlayback` (`3`);
playback types need a `yyyy-MM-dd HH:mm:ss` window.

**VideoQuality** — `Hd` (`1`, main bitrate) or `Fluent` (`2`, sub bitrate). Fluent
is the right default for many viewers.

**RTSP** — not an API protocol value. A LAN RTSP URL is built by the caller from a
device address, and not every model enables it.

## Control

**PTZ** — pan/tilt/zoom; directions `0..11` (eight moves plus zoom and focus). A
movement continues until an explicit stop with the same direction — there is no
server-side auto-stop. → `src/ptz.rs`.

**mirror** — image flip, a display transform rather than a PTZ move.

**preset** — stored position: add (returns an index), recall, clear.

**capture** — a snapshot *request*. Fire-and-forget: no URL is returned. →
[use case 4](02-use-cases.md#4-snapshots).

## Alarms

**AlarmItem** — one `/api/lapp/alarm/list` row: picture URL, type, time, channel.

**alarm window** — `startTime`/`endTime` are Unix **milliseconds**, maximum span
**7 days**, paginated. Long outages need a cursor. →
[use case 3](02-use-cases.md#3-alarm-pipeline).

**AlarmType / AlarmStatus** — typed filters (`All = -1`; unread/read/all).

## SDK and internals

**PCOpenSDK / OpenNetStream.dll** — the proprietary Windows SDK bound by the
`pc-sdk` feature through `libloading`; loaded at runtime, never bundled. →
[use case 7](02-use-cases.md#7-windows-desktop-preview), `src/pc_sdk.rs`.

**HWND preview** — SDK real-play renders into a caller-supplied window handle; it
cannot feed a GPU texture, so it is a preview surface only.

**envelope** — `{code, msg, data?, page?}`; `code` may be a string or a number.
→ `src/types.rs`.

**`10002` replay** — a call rejected with "token expired" is refreshed and replayed
**once** inside the client. → `src/client.rs`.

**`EzvizError`** — `Api { code, msg }`, `Http`, `Json`, `Io`, and (with `pc-sdk`)
`Sdk`. The `Api` variant preserves the platform's own words. → `src/error.rs`.

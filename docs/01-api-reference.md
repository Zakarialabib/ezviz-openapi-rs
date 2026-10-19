# API reference

Every claim here was checked against `src/`. If this document and the source
disagree, the source is right and this file is a bug.

## Hosts and regions

| Constant | Value | Notes |
|---|---|---|
| `DEFAULT_BASE_URL` | `https://isgpopen.ezvizlife.com` | international host |
| `EZVIZ_APP_KEY` / `EZVIZ_APP_SECRET` | env, examples only | never hard-coded in the crate |

A token response carries `areaDomain` — the region-scoped host the token is
valid against. `refresh_token()` calls `set_base_url(areaDomain)` when it is
present, so the client follows the region automatically. Calling the wrong host
with a valid token fails; this is a region mismatch, not a credential problem.

## Response envelope

```json
{ "code": "200", "msg": "操作成功", "data": …, "page": { "total": 1, "page": 0, "size": 10 } }
```

- `code` — `ApiCode` accepts `"200"` (string) and `200` (integer), because
  regions differ. `200` means success.
- `msg` — optional status message.
- `data` — absent on pure-ack endpoints; that is why `call_ack()` exists.
- `page` — present on paged lists only.

`EzvizError::Api { code, msg }` is returned for any non-`200` code, so the
platform's own wording reaches the caller unchanged.

## Token lifecycle

`POST /api/lapp/token/get` with `appKey`, `appSecret`.

1. `access_token()` returns the cached token while it is younger than **24 h**
   (`TOKEN_TTL` in `src/client.rs`).
2. On miss it calls `refresh_token()`; the new token replaces the cached one and
   `areaDomain` (when present) replaces the base URL.
3. Any token-bearing call that comes back `10002` triggers **one** refresh and a
   single replay of the request, inside `post_form()`. A second `10002` is
   returned to the caller.
4. `EzvizClient::with_token(token, area_domain)` skips minting entirely — the
   server-side broker pattern: the broker mints, the client consumes.

Tokens are **per appKey per region**, not per device or per caller. Rotating
them per consumer gains nothing and multiplies `10007`/`10029` rate-limit
failures at fleet scale.

## Endpoints

### Devices

| Method | Endpoint | Parameters | Returns |
|---|---|---|---|
| `list_devices(page_start, page_size)` | `POST /api/lapp/device/list` | `pageStart` (from 0), `pageSize` (≤ 50), access token | `ListResponse<DeviceInfo>` |
| `device_info(serial)` | `POST /api/lapp/device/info` | `deviceSerial` | `DeviceDetail` |
| `device_status(serial)` | `POST /api/lapp/device/status` | `deviceSerial` | `DeviceDetail` (status fields) |
| `list_cameras()` | `POST /api/lapp/camera/list` | access token | `Vec<CameraChannel>` |
| `device_cameras(serial)` | `POST /api/lapp/device/camera/list` | `deviceSerial` | `Vec<CameraChannel>` |

`DeviceInfo` carries `device_serial`, `device_name`, `status`, `device_type`,
`is_encrypt`; `CameraChannel` adds `channel_no` and `video_level`.
Every field that the platform may omit is `Option<…>` — a missing field is not
a parse error.

### Live and playback addresses

| Method | Endpoint | Parameters |
|---|---|---|
| `live_address(serial, ch, protocol, quality, live_type, expire, code, start, stop)` | `POST /api/lapp/live/address/get` | see below |
| `live_hls_address(serial, ch, quality)` | same | shortcut: `protocol=Hls`, `type=Preview` |
| `local_playback_address(serial, ch, quality, start, stop)` | same | `type=LocalPlayback` + window |
| `cloud_playback_address(serial, ch, quality, start, stop)` | same | `type=CloudPlayback` + window |
| `disable_live_address(serial, ch, url_id)` | `POST /api/lapp/live/address/disable` | revokes one issued URL |

Parameter rules:

- `protocol` — `Ezopen=1`, `Hls=2`, `Rtmp=3`, `Flv=4`. **No RTSP value exists
  in this API**; a LAN RTSP URL is constructed by the caller, not minted here.
- `quality` — `Hd=1` (main bitrate) or `Fluent=2` (sub bitrate).
- `type` — `Preview=1`, `LocalPlayback=2`, `CloudPlayback=3`.
- `expireTime` — seconds; values `< 32` are interpreted by the platform as
  "decide for me". Ignored for playback.
- `code` — the device verify code, only for encrypted devices.
- `startTime` / `stopTime` — `yyyy-MM-dd HH:mm:ss`, playback only.

`LiveAddress { id, url, expire_time }`; keep `id` if you intend to revoke the
URL later.

### PTZ

| Method | Endpoint | Parameters |
|---|---|---|
| `ptz_start(serial, ch, direction, speed)` | `POST /api/lapp/device/ptz/start` | `direction` 0–11, `speed` 0/1/2 |
| `ptz_stop(serial, ch, direction)` | `POST /api/lapp/device/ptz/stop` | same direction as the start |
| `ptz_mirror(serial, ch, command)` | `POST /api/lapp/device/ptz/mirror` | `MirrorCommand` |
| `preset_add(serial, ch)` | `POST /api/lapp/device/preset/add` | returns the assigned index |
| `preset_move(serial, ch, index)` | `POST /api/lapp/device/preset/move` | recall |
| `preset_clear(serial, ch, index)` | `POST /api/lapp/device/preset/clear` | delete |

`PtzDirection`: `Up=0 Down=1 Left=2 Right=3 UpperLeft=4 LowerLeft=5
UpperRight=6 LowerRight=7 ZoomIn=8 ZoomOut=9 FocusNear=10 FocusFar=11`.
`PtzSpeed`: `Slow=0 Medium=1 Fast=2`.

A movement is continuous until an explicit `ptz_stop` with the **same**
direction. There is no auto-stop timeout in the API or in this crate.

### Alarms

`list_alarms(start_ms, end_ms, alarm_type, status, page_start, page_size)` →
`POST /api/lapp/alarm/list`.

- `startTime` / `endTime` are **Unix milliseconds**, and the window is capped at
  **7 days**.
- `AlarmType` — `All=-1`, `PirEvent=10000`, `MotionDetection=10002`,
  `VideoTampering=10011`, `VideoLoss=10012`, `LineCrossing=10013`,
  `Intrusion=10014`, `FaceDetection=10015`, `DoorBellRing=10016`,
  `SceneChange=10020`, `Defocus=10021`, `AudioException=10022`,
  `UnattendedBaggage=10023`, `ObjectRemoval=10024`, `IllegalParking=10025`,
  `PeopleGathering=10026`, `Loitering=10027`, `FastMoving=10028`,
  `RegionEntrance=10029`, `RegionExiting=10030`, `LowBattery=10032`,
  `IoAlarm=10100`.
- `AlarmStatus` — `Unread=0`, `Read=1`, `All=2`.
- Paginated: loop on `page` until `items` is short. A polling consumer must
  persist a cursor, not re-query the whole window.

### Capture

`capture(serial, ch)` → `POST /api/lapp/device/capture`. Requests a snapshot and
returns `Result<()>`. **No URL comes back.** The picture is delivered
asynchronously and is fetched from the platform's picture URL (the same one
alarms carry).

## Errors

`EzvizError`:

| Variant | Meaning |
|---|---|
| `Api { code, msg }` | platform returned a non-`200` `code`; the platform's own text is preserved |
| `Http(…)` | transport failure (DNS, TLS, connect, timeout) |
| `Json(…)` | request/response body did not match the expected shape |
| `Io(…)` | local I/O (SDK loading, file paths) |
| `Sdk(String)` | `pc-sdk` only: the native library reported failure |

Platform codes the crate acts on:

| Code | Behaviour |
|---|---|
| `200` | success |
| `10002` | one refresh + one replay inside `post_form()`; a repeat surfaces to the caller |

All other codes are passed through untouched. Vendor documentation lists
`10005`, `10007`, `10029`, `20002`, `20007`, `20014`, `20018` and the `60000`
range for rate limits, missing/offline devices and PTZ failures; the crate does
not reinterpret them, because the numeric ranges differ between regions and
firmware generations. Treat any non-`200` as an API error and branch on the
string if you must.

**The crate never panics in library code.** Lock poisoning is recovered, a
failed client build falls back to a default client, and every fallible path
returns `EzvizError`.

## `pc-sdk` surface (Windows)

`PcSdk::load()` searches `$EZVIZ_SDK_DIR`, then the working directory, then
`PATH`, for `OpenNetStream.dll`. All calls are `&self` and return
`Result<…>` with `EzvizError::Sdk`.

| Method | Purpose |
|---|---|
| `init(app_id)` / `init_lib(auth_addr, platform, app_id)` | library init |
| `fini_lib()` | teardown |
| `set_access_token(token)` | hand the HTTP token to the SDK |
| `set_app_id(app_id)`, `set_client_ver(v)` | identity |
| `set_config_info(key, value)` | numeric config (`CONFIG_*` constants) |
| `set_message_callback(Option<MessageHandler>)` | async messages |
| `alloc_session_ex(private, hwnd)` / `free_session(session)` | P2P session lifecycle |
| `set_session_config(session, key, value)` | per-session config |
| `start_real_play_ex(session, hwnd, serial, channel, safe_key)` | render into a window handle |
| `stop_real_play_ex(session)` | stop rendering |
| `ptz_ctrl_ex(session, serial, channel, command, action, speed)` | PTZ straight over P2P |

Return codes: `PTZCTRL_SUCCESS = 46`, `PTZCTRL_FAILED = 47`,
`PTZ_EXCEPTION = 19`. The window-handle preview is incompatible with a GPU
compositor that expects textures — it is an admin/editor surface, not a player
surface.

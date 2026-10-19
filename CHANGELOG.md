# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-19

### Added

- **Access tokens** — `access_token()` / `refresh_token()` against
  `/api/lapp/token/get`, with a one-shot refresh-and-retry when a call returns
  `10002` (token expired).
- **Devices** — `list_devices()`, `device_info()`, `device_status()`,
  `list_cameras()`, `device_cameras()`.
- **Live and playback addresses** — `live_address()` (protocol, quality, type,
  expiry, verify code, playback window) and the `live_hls_address()` shortcut;
  `disable_live_address()` revokes an issued URL.
- **PTZ** — `ptz_start()` / `ptz_stop()` with 12 directions and three speeds,
  mirroring, and the preset add/move/clear trio.
- **Alarms** — `list_alarms()` over a millisecond timestamp window (7-day
  maximum), filtered by `AlarmType` and `AlarmStatus`, paginated.
- **Capture** — `capture()` requests a device-side snapshot (fire-and-forget;
  the picture is fetched from the platform's picture URL).
- **Optional Windows FFI** — `pc-sdk` feature binds the proprietary
  PCOpenSDK (`OpenNetStream.dll`) through `libloading` for low-latency HWND
  preview. The DLL is never bundled or redistributed.
- MIT license, CI (rustfmt, clippy, test on Linux+Windows, rustdoc) and a
  release workflow.

### Known limitations

- The HTTP verify-code error codes for encrypted devices and the `code`
  parameter required by encrypted HLS are unconfirmed; they need empirical
  verification against real hardware.
- `EZPCOpenSDK_v5.1.18/` is gitignored: the crate binds the SDK but ships no
  part of it.

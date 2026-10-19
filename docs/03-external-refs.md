# External references

Where the facts in [`01-api-reference.md`](01-api-reference.md) come from, what
the optional SDK provides, and when reverse engineering is the right tool
instead of reading documentation.

## 1. EZVIZ Open Platform (primary source)

| Reference | What it gives |
|---|---|
| `https://open.ys7.com/help/en/*` | endpoint catalogue, parameters, response envelope, error codes for the `/api/lapp/**` surface |
| `https://isgpopen.ezvizlife.com` | the international API host this crate defaults to (`DEFAULT_BASE_URL`) |
| `areaDomain` in the token response | the regional host a token is actually valid against |

Everything the crate implements comes from these endpoints. Where the vendor
documentation is ambiguous — numeric error ranges differing by region, whether
encrypted HLS needs the `code` parameter — this crate passes the platform's
`code`/`msg` through unchanged rather than inventing its own mapping.

## 2. PCOpenSDK (optional native path)

The vendor ships a Windows C SDK. Its headers are the origin of every FFI
signature in `src/pc_sdk.rs`:

| File | Content |
|---|---|
| `inc/OpenNetStreamInterFace.h` | exported functions (`OpenSDK_Init`, `OpenSDK_StartRealPlayEx`, `OpenSDK_PTZCtrlEx`, …) |
| `inc/OpenNetStreamDefine.h` | `OPENSDK_RESULT` and config enum values |
| `inc/OpenNetStreamError.h`, `inc/INS_ErrorCode.h` | return codes (`46` success, `47` failure, `19` exception) |
| `lib/win64/OpenNetStream.dll` | runtime library, loaded with `libloading` |

These files are **not** in the repository and not in the published crate
(`EZPCOpenSDK_v5.1.18/` is gitignored; `cargo package --list` contains zero SDK
files). They stay in the working tree so the bindings remain auditable against
their source, and so a developer can run `--features pc-sdk` locally.

## 3. Reverse engineering (fallback, not a dependency)

REA ("Reverse Engineer Anything", `morluto/rea`, npm `rea-agents`, MIT) is a
local MCP server + CLI that inspects binaries, apps and runtime behaviour and
returns code plus evidence. In this environment it is wired as the `rea` MCP
server; roughly 200 tools are exposed, spanning:

- **native** — pseudocode/assembly/strings/call graphs (Hopper, Ghidra, IDA);
- **JavaScript/Electron** — modules, source maps, IPC (static, no engine);
- **.NET / Android / Flutter / firmware / WASM / EVM** — static analysis;
- **runtime** — browser scenarios, ADB, process capture, pcap, signal capture.

Engines are opt-in through env vars (`GHIDRA_INSTALL_DIR`,
`HOPPER_LAUNCHER_PATH`, `REA_IDA_MCP_CONFIG`, `REA_JADX_MCP_JAR`); static JS and
.NET analysis need none.

### Why the crate is not built on REA

The platform API is documented and the SDK ships its headers, so the crate was
built from primary sources. Reverse engineering is reserved for the case that
documentation cannot settle: a protocol that behaves differently from its spec,
or an undocumented payload the official apps send.

| Need | Tool |
|---|---|
| Control a camera (PTZ, live URL, presets, alarms) | this crate |
| Low-latency Windows preview/PTZ | this crate, `pc-sdk` feature |
| Confirm what a vendor app actually sends | REA (static APK/native analysis) |
| Verify an undocumented SDK behaviour | REA against `OpenNetStream.dll` |

## 4. Related crate

The signage platform's TV-control workspace (`tv-rs`) is a sibling project that
follows the same package standard; it consumes this crate for its CCTV widget.
Its `docs/00-rust-package-standard.md` is byte-identical to
[`00-rust-package-standard.md`](00-rust-package-standard.md) here.

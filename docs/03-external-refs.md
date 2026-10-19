# External references

Where the facts in [`01-api-reference.md`](01-api-reference.md) come from, what
the optional native SDK provides, and what to read next.

## Platform documentation

| Reference | What it gives |
|---|---|
| `https://open.ys7.com/help/en/*` | endpoint catalogue, parameters, response envelope, error codes for the `/api/lapp/**` surface |
| `https://isgpopen.ezvizlife.com` | the international host this crate defaults to (`DEFAULT_BASE_URL`) |
| `areaDomain` in the token response | the regional host a token is actually valid against |

Everything implemented here comes from those endpoints. Where the vendor
documentation is ambiguous — numeric error ranges differing by region, whether
encrypted HLS needs the `code` parameter — the crate passes the platform's
`code`/`msg` through unchanged rather than inventing its own mapping, and the
open questions are recorded in the README's limitations.

## The native SDK (optional)

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
their source and so `--features pc-sdk` can be exercised locally.

## Prior art and neighbours

| Project | Why it is worth reading |
|---|---|
| `ezviz-openapi` (this crate) | the HTTP path, typed, with region handling |
| EZVIZ's own mobile SDKs | the reference for behaviour the HTTP API does not document |
| `tv-rs` | the same package standard applied to TV control; useful when a project needs both cameras and screens |

## Reading the code

| If you want to understand | Read |
|---|---|
| how a request is signed and replayed | `src/client.rs` (`post_form`, `call`, `call_ack`) |
| what the envelope can look like | `src/types.rs` (`ApiCode`, `ApiResponse`, `ListResponse`) |
| how regions are followed | `src/token.rs` + `set_base_url` in `src/client.rs` |
| why an error is opaque | `src/error.rs` (`EzvizError::Api` preserves `code`/`msg`) |

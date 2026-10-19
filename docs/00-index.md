# Documentation

One crate, one platform, one idea: wrap the EZVIZ API's quirks once so an
application never rediscovers them. What follows is the long version.

## Read in this order

| # | Page | What it answers |
|---|---|---|
| 0 | [`glossary.md`](glossary.md) | **The map.** Every term, plus the page or source file that explains it. Start here if you have five minutes. |
| 1 | [`01-api-reference.md`](01-api-reference.md) | Which endpoints exist, what each parameter means, what the codes mean, what the SDK surface offers. |
| 2 | [`02-use-cases.md`](02-use-cases.md) | How the crate is actually assembled into a product: live wall, alarm pipeline, token broker, snapshots, playback, desktop preview. |
| 3 | [`03-external-refs.md`](03-external-refs.md) | Official platform docs, the native SDK, and the sibling Rust project. |
| 4 | [`04-integration-prompts.md`](04-integration-prompts.md) | Copy-paste prompts for each of those scenarios, with the pitfalls and the definition of done. |
| — | [`00-rust-package-standard.md`](00-rust-package-standard.md) | The engineering standard this crate follows; reusable outside this repo. |

## Conventions used here

- **Claims about code are checked against code.** If a page names a function, it
  exists in `src/`. When the two disagree, the source is right and the page is a
  bug.
- **A limitation is written down, not hidden.** Unconfirmed platform behaviour is
  labelled unconfirmed, and code that cannot be verified fails loudly rather than
  guessing.
- **No credentials in examples.** `APP_KEY` / `APP_SECRET` are placeholders; the
  real pair belongs on a server.

## Where the code lives

| Path | Contents |
|---|---|
| `src/client.rs` | `EzvizClient`: envelope handling, token cache, the `10002` replay |
| `src/token.rs` | token response and `areaDomain` |
| `src/device.rs`, `live.rs`, `playback.rs`, `ptz.rs`, `alarm.rs`, `capture.rs` | one module per endpoint family |
| `src/types.rs`, `src/error.rs` | envelope/`ApiCode`/paging types, `EzvizError` |
| `src/pc_sdk.rs` | Windows FFI (`pc-sdk` feature only) |
| `examples/` | runnable programs |
| `scripts/release.sh` | the release preflight and publish |

## Absent by design

No `00-naming-conventions.md` and no state matrix: a single crate has no
workspace layout to name, and its status lives in the rustdoc,
`01-api-reference.md` and `CHANGELOG.md`. If a second feature surface ever needs
its own version or a conflicting dependency, the crate splits into a workspace and
those documents appear then.

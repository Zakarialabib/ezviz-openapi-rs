# Contributing

## Prerequisites

- Rust 1.75 or newer.
- `rustfmt` and `clippy` components — CI enforces both, and a host without them
  produces a passing-looking local run that CI then rejects.

```bash
rustup component add rustfmt clippy
```

## Gates

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The `pc-sdk` feature is Windows-only; test it with
`cargo test --features pc-sdk` on Windows. On other platforms the FFI module
compiles to an `Unsupported` error and the SDK tests are skipped.

## Design rules

- **The HTTP API is the portable path.** Every endpoint works with a normal
  appKey/appSecret pair; the SDK is an optional low-latency Windows extra and
  must never become required.
- **No panics in library code.** Blocked or unconfirmed behaviour returns
  `EzvizError::Unsupported` (or a typed API error), never `unwrap()`.
- **Pure functions for wire formats.** Parsing, path building and enum-to-raw
  mapping live in free functions so they test without a socket.
- **Never log or persist credentials.** appKey/appSecret belong to the caller;
  tokens are short-lived and must not be written to disk by this crate.

## Commit convention

[Conventional Commits](https://www.conventionalcommits.org/), one logical change
per commit, scoped by module:

```
feat(live): support encrypted HLS
fix(pc-sdk): gate test import to non-Windows targets
docs: reconcile the CCTV integration spec against src
```

Group related files; never mix formatting churn with behaviour.

## Docs

Start at [`docs/00-index.md`](docs/00-index.md). `docs/01-cctv-integration.md`
describes the signage consumer; its `Source reconciliation` section records
which of its claims were verified against `src/`.

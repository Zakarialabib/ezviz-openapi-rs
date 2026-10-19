#!/usr/bin/env bash
# crates.io release for ezviz-openapi.
#
# Mirrors the tv-rs release flow: verify every gate, refuse a dirty tree for a
# real publish, bump the version in one place, dry-run first.
#
# Usage:
#   scripts/release.sh --dry-run                 # verify everything, publish nothing
#   scripts/release.sh --version 0.2.0 --dry-run # bump first, then verify
#   scripts/release.sh --version 0.2.0           # bump and publish
#   scripts/release.sh --token "$CRATES_IO_TOKEN" --version 0.2.0
#
# Flags:
#   --version X.Y.Z   Bump the crate version in Cargo.toml before packaging.
#   --dry-run         Run `cargo publish --dry-run` (no upload).
#   --token TOKEN     crates.io API token (defaults to $CRATES_IO_TOKEN).
#   --skip-tests      Skip the test/doc gates in the preflight.
#   --yes             Do not prompt before a real publish.
#
# Exit codes: 0 success, 1 preflight failure, 2 bad usage.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

VERSION=""
DRY_RUN=0
TOKEN="${CRATES_IO_TOKEN:-}"
SKIP_TESTS=0
ASSUME_YES=0

usage() {
    sed -n '2,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            [[ $# -ge 2 ]] || { echo "error: --version needs a value" >&2; exit 2; }
            VERSION="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        --token)
            [[ $# -ge 2 ]] || { echo "error: --token needs a value" >&2; exit 2; }
            TOKEN="$2"; shift 2 ;;
        --skip-tests) SKIP_TESTS=1; shift ;;
        --yes|-y) ASSUME_YES=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument '$1'" >&2; usage >&2; exit 2 ;;
    esac
done

step() { printf '\n==> %s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

# --- version bump -----------------------------------------------------------
if [[ -n "$VERSION" ]]; then
    [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].*)?$ ]] \
        || fail "--version must look like 1.2.3 (got '$VERSION')"

    step "Bumping crate version to $VERSION"
    python - "$VERSION" <<'PY'
import re, sys, pathlib
version = sys.argv[1]
path = pathlib.Path("Cargo.toml")
text = path.read_text(encoding="utf-8")
text, count = re.subn(r'(?m)^(version\s*=\s*")[^"]+(")', rf'\g<1>{version}\g<2>', text, count=1)
if count != 1:
    raise SystemExit("could not find the [package] version line")
path.write_text(text, encoding="utf-8")
print(f"package version updated to {version}")
PY
fi

# --- preflight --------------------------------------------------------------
step "Preflight"
[[ -f LICENSE ]] || fail "LICENSE is missing (cargo publish would reject an MIT crate)"
[[ -f README.md ]] || fail "README.md is missing (Cargo.toml declares readme)"
command -v cargo >/dev/null || fail "cargo is not on PATH"

step "cargo fmt --all --check"
cargo fmt --all --check

step "cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

if [[ "$SKIP_TESTS" -eq 0 ]]; then
    step "cargo test"
    cargo test

    step "cargo doc --no-deps (RUSTDOCFLAGS=-D warnings)"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
fi

# --- publish ----------------------------------------------------------------
PUBLISH_ARGS=()
if [[ "$DRY_RUN" -eq 1 ]]; then
    # A dry run uploads nothing, so an uncommitted tree is expected while the
    # release metadata is being iterated on.
    PUBLISH_ARGS+=(--dry-run --allow-dirty)
else
    if [[ -n "$(git status --porcelain)" ]]; then
        fail "working tree is dirty — commit before publishing (a dirty publish cannot be reproduced)"
    fi
    if [[ -z "$TOKEN" ]]; then
        fail "no crates.io token: pass --token or set CRATES_IO_TOKEN"
    fi
    PUBLISH_ARGS+=(--token "$TOKEN")
    if [[ "$ASSUME_YES" -eq 0 ]]; then
        read -r -p "Publish ezviz-openapi to crates.io? [y/N] " reply
        [[ "$reply" =~ ^[Yy]$ ]] || fail "aborted by user"
    fi
fi

step "cargo publish ${PUBLISH_ARGS[*]}"
cargo publish "${PUBLISH_ARGS[@]}"

if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '\ndry run only — nothing was uploaded\n'
else
    printf '\npublished ezviz-openapi; tag the commit (v<version>) to keep the release workflow in sync\n'
fi

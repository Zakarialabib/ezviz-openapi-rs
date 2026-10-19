# Documentation map

`ezviz-openapi` is one crate with one optional feature, so the doc set is
deliberately small. Read in this order:

| # | File | Answers |
|---|---|---|
| 1 | [`../README.md`](../README.md) | what the crate does, install, quick start, feature flags, limitations |
| 2 | [`01-api-reference.md`](01-api-reference.md) | which endpoints exist, what each parameter means, what the codes mean |
| 3 | [`glossary.md`](glossary.md) | what appKey / areaDomain / deviceSerial / channelNo / ezopen actually mean |
| 4 | [`02-cctv-integration.md`](02-cctv-integration.md) | how a signage platform consumes this crate: live widget, alarm interrupt, broker, phasing |
| 5 | [`03-external-refs.md`](03-external-refs.md) | official platform docs, the PCOpenSDK, and the reverse-engineering option |
| 6 | [`04-integration-prompts.md`](04-integration-prompts.md) | ready-to-paste prompts for each integration scenario (live widget, broker, PTZ, alarms, snapshots, NVR grid, playback, Windows preview, tests) |
| — | [`00-rust-package-standard.md`](00-rust-package-standard.md) | the engineering standard this crate follows (shared verbatim with `tv-rs`) |

## Conventions

- **`01-` is the crate's own truth.** Everything in `01-api-reference.md` is
  checked against `src/`; if the two disagree, the source wins and the document
  is wrong.
- **`02-` and beyond describe consumers and tools.** They may reference
  artefacts that live in another repository (the signage platform specs). Those
  references are external and intentionally unresolved here.
- **`glossary.md` is durable.** Definitions are not versioned with the crate;
  they describe the platform's vocabulary.

## Absent by design

There is no `00-naming-conventions.md` and no `03-state-matrix.md`:

- a single crate has no workspace layout to name, and the module layout rule is
  already stated in `CONTRIBUTING.md` and `00-rust-package-standard.md`;
- status lives in the crate-level rustdoc, `01-api-reference.md` and
  `CHANGELOG.md` — a matrix that repeats them would be a fourth copy to drift.

If a second feature surface ever earns its own version or a conflicting
dependency, this crate splits into a workspace and both documents get added at
that point (see `00-rust-package-standard.md` §1 for the decision rule).

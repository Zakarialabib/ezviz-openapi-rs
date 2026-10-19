# ezviz-openapi — docs

Numbering follows the shared convention used across the tv-rs family
(`00-` conventions/standards, `01-` specs/playbook/API reference,
`02-` external references, `03-` state matrix; unnumbered for durable
references).

| File | Role |
| --- | --- |
| [`00-rust-package-standard.md`](00-rust-package-standard.md) | **The reusable engineering standard.** Repository shape, facade pattern, crate-root and error contracts, testability, naming, docs structure, dependency policy, CI gates, publishing. Project-agnostic; an identical copy lives in `tv-rs`. |
| [`01-api-reference.md`](01-api-reference.md) | Authoritative map of the implemented HTTP surface: hosts, endpoint tables, error codes, PCOpenSDK bindings, caveats. |
| [`01-cctv-integration.md`](01-cctv-integration.md) | How this crate is consumed by the signage platform: live widget, alarm interrupt, snapshot, token broker, phasing. The one document here that describes a *consumer* rather than the crate — its platform-side references (SPEC-001…008, `SIGNAGE_SPEC_PACK_v1.md`, `research/`) live in the signage repository. |
| [`02-external-refs.md`](02-external-refs.md) | External tooling and SDK references (REA MCP), plus the decision to build on this crate rather than on REA. |
| [`glossary.md`](glossary.md) | Durable terminology: credentials, devices/channels, streaming protocols, PTZ, alarms and SDK terms. |

## Absent by design

There is no `03-` state matrix and no `00-naming-conventions.md` in this
repository. This is a single crate with one optional feature, so there is no
workspace layout to document and status tracking lives in the crate-level rustdoc
and `01-api-reference.md`. The standard's §1 decision rule is what makes that the
right call rather than an omission — see `00-rust-package-standard.md` §1.

If this crate ever gains a second feature surface that needs independent
versioning or a conflicting dependency, revisit §1 and add the matching
`00-naming-conventions.md` and `03-matrix.md` at that point.

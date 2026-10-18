# External References — REA MCP tooling and the integration decision

Scope: how the REA reverse-engineering MCP server is used, what it gave this
project, and why the EZVIZ CCTV integration is built on the in-house Rust crate
(`ezviz-openapi`) instead of on REA. For the official EZVIZ endpoints and hosts
this crate targets, see `01-api-reference.md`.

---

## 1. What REA is

REA ("Reverse Engineer Anything", `morluto/rea`, npm **`rea-agents`**, MIT) is a
local MCP server + CLI that lets an agent inspect binaries, applications and
runtime behavior down to the code level. It exposes **207 MCP tools** across:

- **Native binaries** — pseudocode, assembly, strings, symbols, calls (via
  Hopper / Ghidra / IDA).
- **JavaScript / Electron** — modules, imports, source maps, IPC, native add-ons
  (static, no engine needed).
- **.NET / Android / Flutter / firmware / WASM / EVM** — static analysis.
- **Runtime** — browser scenarios, ADB devices, process capture, packet/pcap,
  signal recordings.

Analysis runs locally; every result carries its evidence and limitations.

## 2. How to use it — the 5W1H

| | |
| --- | --- |
| **What** | An MCP server + CLI that reverse-engineers a target and returns code/evidence to the agent. |
| **Why** | Understand how a feature works without source; recover pseudocode/assembly; build a similar feature. |
| **Where** | Registered in Hermes as the `rea` MCP server (stdio, `rea mcp`); CLI via `rea <command>`; analysis engines configured by env vars. |
| **When** | When official docs are insufficient and you must reverse-engineer a target (binary, app, protocol). |
| **Who** | The agent (through MCP tools) or the operator (through the CLI). |
| **How** | See below. |

### How — commands

```bash
rea doctor                          # readiness: node, host, engines, skill
rea setup                           # register agents + install engines (Hopper/Ghidra/IDA/JADX)
rea mcp                             # stdio MCP server (already wired in Hermes)
rea analyze-javascript-application <path> --json
rea decompile <target>              # read one part of an app as code
rea function <addr>                 # one function with evidence
rea capabilities                    # list providers + side effects
```

Engines are opt-in via env vars: `GHIDRA_INSTALL_DIR`, `HOPPER_LAUNCHER_PATH`,
`REA_IDA_MCP_CONFIG`, `REA_JADX_MCP_JAR`. Static JS/.NET analysis needs none.

## 3. How we benefited from it (honest)

- **Installed and wired**: `rea-agents@6.5.0` installed globally, registered in
  Hermes as `rea`, **207/207 tools enabled** (full orchestration). Tools load on
  the next session.
- **For the EZVIZ integration we did not need REA.** The Open Platform HTTP API
  is officially documented (`open.ys7.com/help/en/*`) and the PCOpenSDK ships C
  headers (`EZPCOpenSDK_v5.1.18/inc/`). The Rust crate was built from those
  primary sources, not from reverse engineering.
- **REA's real value here is the fallback.** If the SDK or any target has
  undocumented behavior, REA can reverse-engineer it (native analysis via
  Ghidra/Hopper/IDA, or static JS/.NET/Android). It is ready but not required
  for the current path.

## 4. What's left from the docs (unused capability)

- **Native analysis engines not yet configured**: Hopper, Ghidra
  (`GHIDRA_INSTALL_DIR`), IDA (`REA_IDA_MCP_CONFIG`), JADX (`REA_JADX_MCP_JAR`).
- **The REA skill install** (`rea setup`) for agent workflow instructions.
- **Full tool catalog not exercised**: browser capture, ADB device analysis,
  firmware (Binwalk/Unblob), process capture, packet/pcap analysis, signal
  recordings, WASM/EVM inspection.

## 5. The decision: use the in-house Rust crate, not REA

The EZVIZ CCTV integration is built on **`ezviz-openapi`** (Rust, in-house,
tested) at `C:\laragon\www\ezviz-openapi-rs` — not on REA.

It mirrors the SDK's communication protocols:

- **HTTP client** matches the official Open Platform API exactly — same
  `POST application/x-www-form-urlencoded` calls, same endpoints/params/envelope
  (`{code,msg,data,page}`, success `code=="200"`). This is the same protocol
  the official EZVIZ SDKs speak.
- **`pc-sdk` feature** binds the native PCOpenSDK (`OpenNetStream.dll`) via
  FFI (`libloading`, runtime-loaded) — the same protocol the native SDK speaks,
  for low-latency desktop preview/PTZ.

Working + tested: 5 wiremock integration tests + doc-test pass, clippy clean,
`cargo build --features pc-sdk` compiles, `cargo publish --dry-run` passes.

**REA's role is limited**: only if we need something it does that the crate
doesn't — reverse-engineering an undocumented target. Use only the part that
works. Example: if the SDK's protocol needs analysis, point REA at the DLL with
Ghidra. Do not build the integration on REA.

## 6. When to reach for REA vs the crate

| Need | Tool |
| --- | --- |
| Control an EZVIZ cam (PTZ, live, presets, alarms) | `ezviz-openapi` crate |
| Production CCTV feature in the signage app | `ezviz-openapi` crate |
| Understand an undocumented binary / SDK behavior | REA (Ghidra/Hopper/IDA) |
| Static analysis of a JS / .NET / Android app | REA |
| Low-latency desktop preview + PTZ (Windows) | `ezviz-openapi` `pc-sdk` feature |

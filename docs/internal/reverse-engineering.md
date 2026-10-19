# Reverse engineering notes (internal)

Not linked from the documentation index. Kept here because the notes are useful
when documented behaviour and observed behaviour diverge; they are not part of the
crate's public story.

## Tooling

REA ("Reverse Engineer Anything", `morluto/rea`, npm `rea-agents`, MIT) is a
local MCP server + CLI that inspects binaries, applications and runtime behaviour
and returns code plus evidence. Roughly 200 tools across:

- **native** — pseudocode/assembly/strings/call graphs (Hopper, Ghidra, IDA);
- **JavaScript/Electron** — modules, source maps, IPC (static, no engine);
- **.NET / Android / Flutter / firmware / WASM / EVM** — static analysis;
- **runtime** — browser scenarios, ADB, process capture, pcap, signal capture.

Engines are opt-in through env vars (`GHIDRA_INSTALL_DIR`,
`HOPPER_LAUNCHER_PATH`, `REA_IDA_MCP_CONFIG`, `REA_JADX_MCP_JAR`); static JS and
.NET analysis need none.

## When it is the right tool

Only when the primary sources cannot settle a question:

| Question | Tool |
|---|---|
| Control a camera (PTZ, live URL, presets, alarms) | this crate |
| What does the vendor app actually send on this endpoint? | REA (static APK/native analysis) |
| Does `OpenNetStream.dll` behave as its headers claim? | REA against the DLL |
| What are the verify-code error codes on the HTTP path? | REA against the app, then confirm on hardware |

## Open questions that would justify it

1. The `code` parameter required by encrypted HLS, and the HTTP-path equivalents
   of the mobile-SDK `400035`/`400036` verify-code errors.
2. Whether the platform ever returns an integer `code` where the international
   host returns a string (the crate accepts both, but the reason is unconfirmed).
3. Any region-specific field that `areaDomain` switches but the schema does not
   describe.

Write down what the probe proved, not what it looked like: a decompiled
`TrustManager` is evidence about one APK version, not a protocol guarantee.

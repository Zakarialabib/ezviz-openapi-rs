# Security policy

## Reporting a vulnerability

Do not open a public issue. Report privately through GitHub's
[Security Advisories](https://github.com/Zakarialabib/ezviz-openapi-rs/security/advisories/new).

Include the affected version, the endpoint or FFI call involved, and a
reproduction. Expect an acknowledgement within 72 hours.

## Scope

This crate handles cloud credentials and an optional Windows FFI binding.
Reports in these areas are especially relevant:

- **Credential handling** — appKey/appSecret leakage (logging, error messages,
  Debug output), token persistence where the crate should not persist, or a
  credential echoed back in an error.
- **Transport security** — TLS verification is delegated to `rustls`; a path
  that disables verification, or falls back to plaintext, is in scope.
- **FFI safety** — `src/pc_sdk.rs` is the only `unsafe` surface (loading
  `OpenNetStream.dll` and calling its exported functions). Unsound pointer
  handling or a lifetime bug there is in scope.
- **Command injection / path traversal** — anything where an argument is
  interpolated into a URL or file path without encoding.

## Out of scope

- Vulnerabilities in the proprietary PCOpenSDK / `OpenNetStream.dll` — report
  those to the vendor. This crate never redistributes the DLL.
- Credential compromise on the EZVIZ platform itself.
- The unconfirmed verify-code behaviour documented as a known limitation; it is
  a compatibility question, not a vulnerability.

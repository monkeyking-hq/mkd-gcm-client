# Changelog

## 0.2.0

### Added
- **Bearer auth** on `NntpMemoryStore`: `with_bearer_token` / `with_bearer_authzid` → AUTHINFO SASL OAUTHBEARER (named PAT / session).
- **`post_json`**: POST with `Content-Type: application/json` and `X-Mkd-Format: json`.
- **`encode_oauthbearer_message`** (RFC 7628 one-shot, matches `mkd-gcm-server`).
- **`mkd-gcm-ffi`** crate: C ABI (`cdylib`/`staticlib`) — create client, set auth, post / post_json, last_error.
- **`include/mkd_gcm.h`**: public C header.
- **Java thin SDK** (`bindings/java`, Maven `dev.monkeyking:mkd-gcm-sdk`): JNA wrapper, `GcmClient`, `CorrectionSubmission` (camelCase JSON for the browser language-plugin contract).
- **Optional** `mkd-gcm-sdk-jackson2` / `mkd-gcm-sdk-jackson3` modules.
- **`mkd-gcm-natives`**: Maven packaging of host-platform `mkd_gcm_ffi` shared libraries (classpath extraction + platform classifier JARs).
- **Maven Central** publish profile (`-Pcentral`) via Sonatype Central Publisher Portal (`publishingServerId=central`).

### Changed
- Workspace version **0.2.0**.
- **License: MIT**, Copyright (c) 2026 MonkeyKing.dev (this repository / client SDKs).
- Public docs use **GCM** (not NNTP) for application consumers; no customer-identifying examples.
- Preferred env: `MKD_GCM_URL` / `MKD_GCM_HOST` / `MKD_GCM_PORT` / `MKD_GCM_USER` / `MKD_GCM_PASS` / `MKD_GCM_FROM` (legacy `MKD_NNTP_*` still accepted).
- Removed one-shot multi-repo PR helper script from this public repo (belongs in private ops tooling).
- Public type alias **`GcmMemoryStore`** (= `NntpMemoryStore`) for application-facing docs/API.
- All crate sources SPDX **MIT** / Copyright (c) 2026 MonkeyKing.dev.

## Unreleased (historical)

### Changed

- Server product rename: docs and freeport integration discover **`mkd-gcm-server`** (legacy `mkd-nntp-server` paths still accepted during transition). Env: `MKD_GCM_SERVER_BIN` or `MKD_NNTP_SERVER_BIN`.

## 0.1.0

### Added

- Initial `mkd-gcm` crate: NNTP / GCM memory client.
- `NntpMemoryStore`, `FsNntpMemoryStore`, `MemoryStore` trait.
- `GroupPath`, `HierarchyBranch`, article/post types, env config helpers.
- Optional feature `locator-migrate` (legacy wing/room → group path oracle).
- Freeport integration test against a GCM-compatible server binary.

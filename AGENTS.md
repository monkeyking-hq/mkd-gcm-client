<!--
Copyright (c) 2026 MonkeyKing.dev
SPDX-License-Identifier: MIT
-->

# AI Agent & LLM Guide for mkd-gcm-client

**Golden Circlet Memory (GCM) network** client libraries for remote `mkd-gcm-server`.

**License:** MIT — Copyright (c) 2026 MonkeyKing.dev. All new and updated files
in this repository use MIT. See root `LICENSE`. Do not reintroduce
non-MIT license identifiers in this public client repository.

**Documentation:** Keep product specs and private process notes out of this
public repo’s README. Durable product specs live in company docs; this AGENTS.md
is for agents working on the public client only.

**Public GitHub:** https://github.com/monkeyking-hq/mkd-gcm-client  
Before any `gh` write, confirm active account is **`monkeykinghq`**
(`gh auth status` / `gh auth switch -u monkeykinghq`).

## Confidentiality (HARD RULE)

**NEVER** reveal actual customer or user information in:

- comments, documentation, README examples, commit messages, PR text
- sample group paths, hostnames, emails, tokens, or config snippets
- public or semi-public code

Use only fictional placeholders, e.g.:

- groups: `mkd.gcm.orgs.org.example.language.projects.crowdsource-demo.inbound`
- hosts: `gcm.monkeyking.dev` (public product host) or `gcm.example.internal` (generic)
- emails: `user@example.com`

Do not mention real customer products, private integration targets, or real
tenant domains.

## Scope

- **Rust crate `mkd-gcm`**: GCM client API, `GroupPath`, article types, env
  config (`MKD_GCM_*` preferred), bearer OAUTHBEARER, `post_json`.
- **Rust crate `mkd-gcm-ffi`**: C ABI (`cdylib`) over `mkd-gcm` for thin SDKs.
- **`include/mkd_gcm.h`**: C header for FFI consumers.
- **`bindings/java`**: thin JNA SDK for JVM host applications.
- **Optional** `bindings/java-jackson2` / `java-jackson3` for ObjectMapper helpers.
- Application-facing docs and SDK names say **GCM**. Wire-protocol details are
  internal to this library; do not require app developers to learn NNTP.

## Change Protocol

When changing the public Rust API of `mkd-gcm`, update dependents that consume
the crate on the same feature branch name across repos that depend on it.

1. Checkout main and pull (each affected repo).
2. Create matching feature branches.
3. Bump versions + CHANGELOG entries.
4. Run `cargo test -p mkd-gcm -p mkd-gcm-ffi`.
5. For Java: `mvn -q test` under `bindings/java` (and jackson modules if touched).

Multi-repo PR automation scripts belong in private ops tooling, **not** in this
public client repository.

## Freeport

Any live server integration tests **must** allocate a free port at runtime.
Never hardcode lab ports in tests.

## Packaging notes

- **crates.io:** metadata is set; do not `cargo publish` without an explicit release.
- **Maven groupId:** `dev.monkeyking`. Reactor parent: `bindings/pom.xml`.
- **Maven Central:** profile `-Pcentral` uses Sonatype Central Portal plugin;
  credentials live only in `~/.m2/settings.xml` server id **`mkd-central`**
  (never commit). Other products use separate server ids (e.g. `hashseal-central`).
- **Natives:** module `bindings/java-natives` builds `mkd-gcm-ffi` via cargo and
  packages the shared library; multi-OS classifier JARs need per-platform builds.

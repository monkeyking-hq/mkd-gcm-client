# mkd-gcm-client

[![CI](https://github.com/monkeyking-hq/mkd-gcm-client/actions/workflows/ci.yml/badge.svg)](https://github.com/monkeyking-hq/mkd-gcm-client/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Golden Circlet Memory (GCM) network** client libraries.

This repository is the multi-language client surface for a GCM-compatible server
(`mkd-gcm-server`). Callers use the **GCM** protocol API. Transport details are
an implementation concern of this library and should stay invisible to
application code.

| | |
|--|--|
| **Status** | v0.2.0 — public beta; APIs may evolve before 1.0 |
| **License** | MIT — Copyright (c) 2026 MonkeyKing.dev |
| **Repository** | https://github.com/monkeyking-hq/mkd-gcm-client |

## Layout

```text
crates/
  mkd-gcm/        Rust core (GCM client, GroupPath, post_json, bearer auth)
  mkd-gcm-ffi/    C ABI (cdylib) for thin language SDKs
include/
  mkd_gcm.h       C header for FFI
bindings/
  java/                 Thin JNA SDK (core, zero Jackson)
  java-jackson2/        Optional Jackson 2.x helpers
  java-jackson3/        Optional Jackson 3.x helpers
```

## Rust crate: `mkd-gcm`

```rust
use mkd_gcm::{GcmMemoryStore, GroupPath, MemoryStore, PostHeaders};

let store = GcmMemoryStore::from_url("gcm.monkeyking.dev:1119")?
    .with_bearer_token(std::env::var("MKD_GCM_TOKEN")?);
let group = GroupPath::parse(
    "mkd.gcm.orgs.org.example.language.projects.crowdsource-demo.inbound",
)?;
let pref = store.post_json(
    &group,
    "language correction",
    r#"{"email":"user@example.com","locale":"en","proposedText":"…"}"#,
    &PostHeaders::new(),
)?;
```

| Variable | Role |
|----------|------|
| `MKD_GCM_URL` or `MKD_GCM_HOST`+`MKD_GCM_PORT` | GCM server address |
| `MKD_GCM_USER` / `MKD_GCM_PASS` | Optional password auth |
| `MKD_GCM_FROM` | Default From: mailbox |
| `MKD_GCM_TOKEN` | (app-level) bearer / project_submit PAT |

(`GcmMemoryStore` is the public name for the remote client; an older type
alias remains for existing code. Legacy `MKD_NNTP_*` env names are still
accepted for existing deploys.)

### crates.io

Package metadata is crates.io-ready. Publishing is a separate release step:

```bash
cargo publish -p mkd-gcm
cargo publish -p mkd-gcm-ffi
```

## C ABI: `mkd-gcm-ffi`

```bash
cargo build -p mkd-gcm-ffi --release
```

| OS | Artifact |
|----|----------|
| Linux | `target/release/libmkd_gcm_ffi.so` |
| macOS | `target/release/libmkd_gcm_ffi.dylib` |
| Windows | `target/release/mkd_gcm_ffi.dll` |

Header: [`include/mkd_gcm.h`](include/mkd_gcm.h).

### Installing the native library (JVM / host process)

1. Build `mkd-gcm-ffi` for the target OS/arch (or CI-produce platform packages).
2. Place the shared library on the host:
   - **Linux:** e.g. `/usr/local/lib/` and run `ldconfig`, or ship next to the app
   - **Windows:** directory on `PATH`, or next to the JVM process, or set `jna.library.path`
   - **macOS:** e.g. `/usr/local/lib` or app bundle + `jna.library.path`
3. For Java: `-Djna.library.path=/path/to/dir` **or** system library path.
4. The process that loads the library must match arch (x86_64 vs aarch64).

Do not commit binary artifacts to git; ship via release packages or install scripts.

## Java SDK (thin JNA)

See [`bindings/java/README.md`](bindings/java/README.md).

```java
try (GcmClient c = GcmClient.connect("gcm.monkeyking.dev", 1119)) {
    c.setBearerToken(tokenFromServerSecrets);
    c.postCorrection(inboundGroup, submission); // CorrectionSubmission JSON
}
```

**Host application path:** browser language plugin → `postUrl` REST on your
app server → load GCM token from server secrets → this SDK → GCM network.
The browser never holds the PAT.

Optional JSON: [`java-jackson2`](bindings/java-jackson2) / [`java-jackson3`](bindings/java-jackson3).

### Maven (`dev.monkeyking`)

Java modules live under [`bindings/`](bindings/) with parent
`mkd-gcm-sdk-parent`.

```xml
<dependency>
  <groupId>dev.monkeyking</groupId>
  <artifactId>mkd-gcm-sdk</artifactId>
  <version>0.2.0</version>
</dependency>
<dependency>
  <groupId>dev.monkeyking</groupId>
  <artifactId>mkd-gcm-natives</artifactId>
  <version>0.2.0</version>
</dependency>
```

`mkd-gcm-natives` packages **windows-x86_64** and **linux-x86_64** FFI libraries
in one JAR (Linux `.so` built via WSL when packaging on Windows). The SDK
extracts the matching library at runtime unless you set `jna.library.path`.

**Publish to Maven Central** (Sonatype Central Portal):

1. `~/.m2/settings.xml` server id **`mkd-central`** with a Portal user token  
   (HashSeal and other lines use their own ids, e.g. `hashseal-central`)
2. GPG key available to Maven
3. From `bindings/`: `mvn clean deploy -Pcentral`

See [`bindings/java/README.md`](bindings/java/README.md). Optional profile
`-Pgithub-packages` still targets GitHub Packages.

## Design

One **Rust core**, many **thin SDKs** (Java/Python/.NET via FFI). Applications
talk **GCM**, not a reimplemented wire stack per language.

## Building & testing

```bash
cargo test -p mkd-gcm -p mkd-gcm-ffi
cd bindings/java && mvn -q test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details. Security reports:
[SECURITY.md](SECURITY.md).

## License

MIT License — Copyright (c) 2026 MonkeyKing.dev

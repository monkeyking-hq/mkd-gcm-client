# Contributing to mkd-gcm-client

Thanks for helping improve the Golden Circlet Memory (GCM) client libraries.

## Development setup

### Rust

- Install the toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml) (Rust 1.97.1).
- From the repo root:

```bash
cargo test -p mkd-gcm -p mkd-gcm-ffi
cargo clippy -p mkd-gcm -p mkd-gcm-ffi --all-targets
cargo fmt --all
```

Optional live integration test (skips if the server binary is missing):

```bash
# set MKD_GCM_SERVER_BIN to a mkd-gcm-server executable, or place it on PATH
cargo test -p mkd-gcm --test memory_nntp_integration -- --nocapture
```

### Java

```bash
cd bindings
mvn test          # builds mkd-gcm-ffi (cargo) + all modules
mvn package
```

Optional Jackson helpers are modules of the same reactor. Natives packaging:
`java-natives` → `dev.monkeyking:mkd-gcm-natives`.

Publish to Maven Central (maintainers; needs Portal token + GPG):

```bash
cd bindings
mvn clean deploy -Pcentral
```

## Pull requests

- Keep changes focused and documented in `CHANGELOG.md` when user-visible.
- Do not commit secrets, real tokens, customer hostnames, or personal emails.
- Use fictional placeholders in examples (see root `AGENTS.md` confidentiality rules).
- New/updated source files: MIT SPDX header and Copyright (c) 2026 MonkeyKing.dev.
- Ensure CI is green (Rust + Java jobs).

## License

By contributing, you agree that your contributions are licensed under the MIT License (see [LICENSE](LICENSE)).

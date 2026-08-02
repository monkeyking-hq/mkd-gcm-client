# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.2.x   | Yes       |
| &lt; 0.2 | Best effort |

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Prefer one of:

1. **GitHub Security Advisories** for [monkeyking-hq/mkd-gcm-client](https://github.com/monkeyking-hq/mkd-gcm-client/security/advisories/new)
2. Email **security@monkeyking.dev** with a description, impact, and reproduction steps

We will acknowledge reports as soon as practical and coordinate a fix and disclosure timeline.

## Scope

In scope: this client library (Rust crates, C ABI, Java SDK), credential handling in the client, and packaging scripts in this repository.

Out of scope: third-party GCM servers, host application secrets, and infrastructure not published in this repository.

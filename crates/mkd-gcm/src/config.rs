// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Env config for the production GCM memory client ([`crate::GcmMemoryStore`] /
//! [`crate::NntpMemoryStore`]).
//!
//! Preferred variables: `MKD_GCM_URL` (or `MKD_GCM_HOST`+`MKD_GCM_PORT`);
//! optional `MKD_GCM_USER` / `MKD_GCM_PASS` / `MKD_GCM_FROM` / `MKD_GCM_ROOT`.
//! Legacy `MKD_NNTP_*` names remain accepted for existing deploys.
//!
//! Soft dual-backend vars are no longer supported
//! (`MKD_MEMORY_BACKEND`, `MKD_MEMORY_READ_PRIMARY`, `MKD_MEMORY_NNTP_SOFT`,
//! `MKD_MEMORY_SOFT_SECONDARY`).

use crate::error::{Error, Result};

use crate::MemoryStore;
use crate::nntp::NntpMemoryStore;

/// Resolved memory configuration from the environment.
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Server URL from `MKD_GCM_URL` / `MKD_GCM_HOST`+`MKD_GCM_PORT`
    /// (or legacy `MKD_NNTP_*`).
    pub nntp_url: Option<String>,
    pub nntp_user: Option<String>,
    pub nntp_pass: Option<String>,
    pub default_from: Option<String>,
    pub mkd_nntp_root: Option<String>,
}

impl MemoryConfig {
    pub fn from_env() -> Self {
        memory_config_from_env()
    }
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// Load [`MemoryConfig`] from process environment.
///
/// Preferred public names: `MKD_GCM_URL` / `MKD_GCM_HOST`+`MKD_GCM_PORT` /
/// `MKD_GCM_USER` / `MKD_GCM_PASS` / `MKD_GCM_FROM`.
/// Legacy `MKD_NNTP_*` names remain accepted for existing deploys.
pub fn memory_config_from_env() -> MemoryConfig {
    let nntp_url = env_nonempty("MKD_GCM_URL")
        .or_else(|| env_nonempty("MKD_NNTP_URL"))
        .or_else(|| {
            let host = env_nonempty("MKD_GCM_HOST").or_else(|| env_nonempty("MKD_NNTP_HOST"));
            let port = env_nonempty("MKD_GCM_PORT").or_else(|| env_nonempty("MKD_NNTP_PORT"));
            match (host, port) {
                (Some(h), Some(p)) => Some(format!("{h}:{p}")),
                (Some(h), None) => Some(format!("{h}:1119")),
                (None, Some(p)) => Some(format!("127.0.0.1:{p}")),
                _ => None,
            }
        });

    MemoryConfig {
        nntp_url,
        nntp_user: env_nonempty("MKD_GCM_USER").or_else(|| env_nonempty("MKD_NNTP_USER")),
        nntp_pass: env_nonempty("MKD_GCM_PASS").or_else(|| env_nonempty("MKD_NNTP_PASS")),
        default_from: env_nonempty("MKD_GCM_FROM").or_else(|| env_nonempty("MKD_NNTP_FROM")),
        mkd_nntp_root: env_nonempty("MKD_GCM_ROOT").or_else(|| env_nonempty("MKD_NNTP_ROOT")),
    }
}

/// Build a [`crate::GcmMemoryStore`] from the process environment.
///
/// Returns `Ok(None)` when no GCM URL is configured (caller keeps the
/// historical direct-drawer path). Returns `Err` when the URL is malformed.
pub fn memory_store_from_env() -> Result<Option<std::sync::Arc<dyn MemoryStore>>> {
    let cfg = memory_config_from_env();
    build_memory_store(&cfg)
}

/// Construct a [`crate::GcmMemoryStore`] from an explicit config (tests).
pub fn build_memory_store(cfg: &MemoryConfig) -> Result<Option<std::sync::Arc<dyn MemoryStore>>> {
    let Some(url) = cfg.nntp_url.as_deref() else {
        return Ok(None);
    };
    let mut store = NntpMemoryStore::from_url(url)?;
    if let (Some(u), Some(p)) = (&cfg.nntp_user, &cfg.nntp_pass) {
        store = store.with_auth(u.clone(), p.clone());
    }
    if let Some(from) = &cfg.default_from {
        store = store.with_default_from(from.clone());
    }
    Ok(Some(std::sync::Arc::new(store)))
}

/// Parse-only helper retained for tests that used to round-trip the env value.
#[doc(hidden)]
pub fn _assert_no_legacy_env() -> Result<()> {
    for var in [
        "MKD_MEMORY_BACKEND",
        "MKD_MEMORY_READ_PRIMARY",
        "MKD_MEMORY_NNTP_SOFT",
        "MKD_MEMORY_SOFT_SECONDARY",
    ] {
        if let Ok(v) = std::env::var(var)
            && !v.trim().is_empty()
        {
            return Err(Error::Config(format!(
                "{var}={v} is no longer supported; use MKD_GCM_URL"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_legacy_env_vars() {
        let prev = [
            "MKD_MEMORY_BACKEND",
            "MKD_MEMORY_READ_PRIMARY",
            "MKD_MEMORY_NNTP_SOFT",
            "MKD_MEMORY_SOFT_SECONDARY",
        ]
        .iter()
        .map(|k| (k.to_string(), std::env::var(k).ok()))
        .collect::<Vec<_>>();

        // SAFETY: tests run single-threaded for this env block; restored below.
        unsafe {
            std::env::set_var("MKD_MEMORY_BACKEND", "dual");
            std::env::set_var("MKD_MEMORY_READ_PRIMARY", "nntp");
        }
        assert!(_assert_no_legacy_env().is_err());

        unsafe {
            std::env::remove_var("MKD_MEMORY_BACKEND");
            std::env::remove_var("MKD_MEMORY_READ_PRIMARY");
        }
        assert!(_assert_no_legacy_env().is_ok());

        for (k, v) in prev {
            match v {
                Some(v) => unsafe { std::env::set_var(&k, v) },
                None => unsafe { std::env::remove_var(&k) },
            }
        }
    }
}

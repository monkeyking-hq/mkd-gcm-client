// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! **mkd-gcm** — Golden Circlet Memory (GCM) network client library.
//!
//! Application surface: [`MemoryStore`], [`GcmMemoryStore`] (production remote
//! client), [`GroupPath`], and article types. Talks to a GCM-compatible server
//! (`mkd-gcm-server`). Standalone crate with no shared monorepo runtime.
//!
//! Callers post/read via [`MemoryStore`]. Prefer the public name
//! [`GcmMemoryStore`]; [`NntpMemoryStore`] is the same type (wire-level name
//! kept for existing code). Transport is an implementation detail of this crate.
//!
//! Locators are full [`GroupPath`] strings only. Legacy wing/room mapping is
//! available solely under feature `locator-migrate`.

mod config;
mod error;
mod fs_nntp;
mod nntp;
mod path;
mod types;

#[cfg(feature = "locator-migrate")]
pub mod migrate_map;

pub use config::{MemoryConfig, build_memory_store, memory_config_from_env, memory_store_from_env};
pub use error::{Error, Result};
pub use fs_nntp::FsNntpMemoryStore;
pub use nntp::{NntpMemoryStore, encode_oauthbearer_message};
pub use path::{GroupPath, HierarchyBranch, normalize_task_leaf, sanitize_token};
pub use types::{Article, GroupInfo, OverviewRow, PostHeaders, PostRef, SearchHit};

/// Preferred public name for the production remote GCM memory client.
///
/// Same type as [`NntpMemoryStore`] (historical wire-oriented name).
pub type GcmMemoryStore = NntpMemoryStore;

/// Shared GCM memory client API.
///
/// Sync surface so host applications can call without a Tokio runtime.
/// Network I/O uses blocking TCP; callers on async runtimes should
/// `spawn_blocking` if latency matters.
pub trait MemoryStore: Send + Sync {
    /// Ensure the group exists for subsequent posts.
    ///
    /// Current `mkd-gcm-server` accepts POST to any `mkd.*` name and creates
    /// the storage directory on write. [`NntpMemoryStore::ensure_group`] is
    /// therefore a no-op over the wire. Local FS tooling may still mkdir under
    /// `MKD_GCM_ROOT` / `MKD_NNTP_ROOT` for LIST visibility. Multi-user
    /// deployments may later require authenticated Control `newgroup`.
    fn ensure_group(&self, path: &GroupPath) -> Result<()>;

    /// POST a new article (or revision when Supersedes/slot headers set).
    fn post(&self, group: &GroupPath, headers: &PostHeaders, body: &str) -> Result<PostRef>;

    /// Fetch by Message-ID and/or group+slot.
    fn get_article(&self, post: &PostRef) -> Result<Article>;

    /// List groups matching optional wildmat (`*` only; empty = all known).
    fn list_groups(&self, wildmat: Option<&str>) -> Result<Vec<GroupInfo>>;

    /// Optional XOVER; default empty when the backend does not support it.
    fn over(&self, _group: &GroupPath, _range: Option<&str>) -> Result<Vec<OverviewRow>> {
        Ok(Vec::new())
    }

    /// Optional full-text search; default empty (not all backends implement it).
    fn search(&self, _query: &str) -> Result<Vec<SearchHit>> {
        Ok(Vec::new())
    }

    /// Backend label for logs/metrics (`nntp`, `fs`).
    fn backend_name(&self) -> &'static str;
}

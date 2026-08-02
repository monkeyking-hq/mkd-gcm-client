// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Canonical mkd group path helpers.
//!
//! The **sole public locator** for NNTP-backed memory is [`GroupPath`].
//! Allowed wire roots:
//! - `mkd.<branch>.…` — product ontology (projects/agents/users/system/addressbook)
//! - `monkeyking` / `monkeyking.…` — top-level company hierarchy (sibling of `mkd.*`)
//!
//! There is no dual wing/room product ontology. Legacy wing+room mapping lives
//! only behind the `locator-migrate` feature ([`super::migrate_map`]).
//!
//! Living task collaboration uses the **parent** tasks group
//! `mkd.<branch>.<name>.tasks` with in-group threads (Message-ID / References).
//! [`GroupPath::task_group`] remains only for **legacy** child-leaf paths during
//! migration inventory.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Top-level branch under the wire root `mkd.`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HierarchyBranch {
    Projects,
    Agents,
    Users,
    System,
    Addressbook,
}

impl HierarchyBranch {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Projects => "projects",
            Self::Agents => "agents",
            Self::Users => "users",
            Self::System => "system",
            Self::Addressbook => "addressbook",
        }
    }
}

impl fmt::Display for HierarchyBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Canonical NNTP group path (`mkd.projects.mkd.log`, `monkeyking`, …).
///
/// This is the only public locator type for GCM / mkd group location.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupPath {
    /// Full dotted name including wire root (`mkd.` or `monkeyking`).
    name: String,
}

impl GroupPath {
    /// True when `name` is a locked wire root: `mkd.*` or `monkeyking` / `monkeyking.*`.
    pub fn is_allowed_root(name: &str) -> bool {
        name.starts_with("mkd.") || name == "monkeyking" || name.starts_with("monkeyking.")
    }

    /// Parse a full group name (must use an allowed wire root; non-empty; no whitespace).
    pub fn parse(name: impl AsRef<str>) -> Result<Self> {
        let n = name.as_ref().trim();
        if n.is_empty() {
            return Err(Error::Message("empty group path".into()));
        }
        if !Self::is_allowed_root(n) {
            return Err(Error::Message(format!(
                "group must start with mkd. or be monkeyking / monkeyking.*: {n}"
            )));
        }
        if n.chars().any(|c| c.is_whitespace()) {
            return Err(Error::Message(format!("group has whitespace: {n}")));
        }
        Ok(Self {
            name: n.to_string(),
        })
    }

    /// Construct without validation (tests / known-good strings / migrator).
    pub fn unchecked(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Bare top-level company hierarchy root: `monkeyking`.
    pub fn monkeyking() -> Self {
        Self::unchecked("monkeyking".to_string())
    }

    /// Subgroup under the top-level `monkeyking` root (e.g. `log` → `monkeyking.log`).
    pub fn monkeyking_sub(segments: &str) -> Self {
        let segs = segments.trim().trim_matches('.');
        if segs.is_empty() {
            return Self::monkeyking();
        }
        let cleaned = segs
            .split('.')
            .map(sanitize_token)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(".");
        if cleaned.is_empty() {
            Self::monkeyking()
        } else {
            Self::unchecked(format!("monkeyking.{cleaned}"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// FS relative path under `MKD_NNTP_ROOT`.
    ///
    /// - `mkd.projects.mkd.log` → `projects/mkd/log` (strip `mkd.` wire root)
    /// - `monkeyking` → `monkeyking`
    /// - `monkeyking.log` → `monkeyking/log`
    pub fn fs_relative(&self) -> String {
        let suffix = self.name.strip_prefix("mkd.").unwrap_or(&self.name);
        suffix.replace('.', "/")
    }

    /// Living collab container for a subject: `mkd.<branch>.<name>.tasks`.
    ///
    /// System branch has no name segment: `mkd.system.tasks`.
    pub fn project_tasks(branch: HierarchyBranch, name: &str) -> Self {
        match branch {
            HierarchyBranch::System => Self::unchecked("mkd.system.tasks".to_string()),
            other => {
                let n = sanitize_token(name);
                Self::unchecked(format!("mkd.{other}.{n}.tasks"))
            }
        }
    }

    /// Legacy per-task **child** group: `mkd.<branch>.<name>.tasks.<id>-<slug>`.
    ///
    /// **Deprecated for living collab writes**. Prefer [`project_tasks`] plus
    /// in-group threading. Keep for migrator inventory of historical child leaves.
    #[deprecated(
        note = "living collab uses project_tasks + in-group threads; task_group is legacy inventory only"
    )]
    pub fn task_group(branch: HierarchyBranch, name: &str, task_id: &str, slug: &str) -> Self {
        let n = sanitize_token(name);
        let id = sanitize_token(task_id.trim().trim_start_matches("task-"));
        let s = sanitize_token(slug);
        let leaf = if s.is_empty() || s == "unknown" {
            id.to_string()
        } else {
            format!("{id}-{s}")
        };
        match branch {
            HierarchyBranch::System => Self::unchecked(format!("mkd.system.tasks.{leaf}")),
            other => Self::unchecked(format!("mkd.{other}.{n}.tasks.{leaf}")),
        }
    }

    /// True when `group` looks like a **legacy** per-task child under `…tasks.<leaf>`
    /// (extra segment after `.tasks`), not the bare parent `…tasks` group.
    pub fn is_legacy_task_child_group(group: &str) -> bool {
        let g = group.trim();
        if g == "mkd.system.tasks" || g.ends_with(".tasks") {
            // bare parent: ends with .tasks and has no further segment after tasks
            if g.ends_with(".tasks") {
                // if there's content after ".tasks." it's a child
                if let Some(idx) = g.rfind(".tasks") {
                    let after = &g[idx + ".tasks".len()..];
                    return after.starts_with('.') && after.len() > 1;
                }
            }
        }
        // also: ends with .tasks.<something>
        g.contains(".tasks.")
    }
}

impl fmt::Display for GroupPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl FromStr for GroupPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl AsRef<str> for GroupPath {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

/// Sanitize a path segment token for NNTP group segments.
pub fn sanitize_token(s: &str) -> String {
    let s = s.trim().to_lowercase();
    let s = s.replace(['_', ' '], "-");
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
            out.push(c);
        } else {
            out.push('-');
        }
    }
    let collapsed = {
        let mut r = String::new();
        let mut prev_dash = false;
        for c in out.chars() {
            if c == '-' {
                if !prev_dash {
                    r.push(c);
                }
                prev_dash = true;
            } else {
                r.push(c);
                prev_dash = false;
            }
        }
        r.trim_matches(|c| c == '-' || c == '.').to_string()
    };
    if collapsed.is_empty() {
        "unknown".into()
    } else {
        collapsed
    }
}

/// Normalize a legacy per-task room/leaf name to the `<id>-<slug>` form.
///
/// Accepts inputs produced by the previous `task-<id>` room naming and by
/// the older `merge-<id>` / `merge-task-<id>` aliases. Used by the migration
/// oracle and by callers that still receive historical task leaf tokens.
pub fn normalize_task_leaf(room: &str, slug: Option<&str>) -> Option<String> {
    let mut r = sanitize_token(room);
    if r.is_empty() || matches!(r.as_str(), "." | "general" | "overview" | "main") {
        return None;
    }
    if r == "tasks" {
        return None;
    }
    if let Some(rest) = r.strip_prefix("merge-task-") {
        r = format!("task-{rest}");
    } else if let Some(rest) = r.strip_prefix("merge-") {
        r = format!("task-{rest}");
    }
    while r.starts_with("task-task-") {
        r = format!("task-{}", &r[10..]);
    }
    if r.starts_with("task-") && r.len() > 5 {
        while r.contains("task-task-") {
            r = r.replace("task-task-", "task-");
        }
        let id = &r[5..];
        if id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()) {
            let id = id.to_string();
            return Some(match slug {
                Some(s) => {
                    let s = sanitize_token(s);
                    if s.is_empty() || s == "unknown" {
                        id
                    } else {
                        format!("{id}-{s}")
                    }
                }
                None => id,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_empty() {
        assert!(GroupPath::parse("").is_err());
        assert!(GroupPath::parse("   ").is_err());
    }

    #[test]
    fn parse_rejects_non_allowed_root() {
        assert!(GroupPath::parse("projects.mkd").is_err());
        assert!(GroupPath::parse("mkd").is_err());
        assert!(GroupPath::parse("Mkd.projects.mkd").is_err());
        assert!(GroupPath::parse("other.top").is_err());
        assert!(GroupPath::parse("MonkeyKing").is_err());
    }

    #[test]
    fn parse_rejects_whitespace() {
        assert!(GroupPath::parse("mkd.projects.mkd log").is_err());
        assert!(GroupPath::parse("mkd.projects.mkd\tlog").is_err());
        assert!(GroupPath::parse("monkeyking log").is_err());
    }

    #[test]
    fn parse_accepts_representative_paths() {
        let cases = [
            "mkd.projects.mkd.log",
            "mkd.agents.grok.conversations",
            "mkd.users.nate.log",
            "mkd.system.rehearsal",
            "mkd.addressbook.contacts.main",
            "mkd.projects.mkd.tasks.1780536515909-fix-nntp-drift",
            "monkeyking",
            "monkeyking.log",
            "monkeyking.projects.specs",
        ];
        for c in cases {
            let g = GroupPath::parse(c).unwrap_or_else(|e| panic!("parse {c}: {e}"));
            assert_eq!(g.as_str(), c);
            assert_eq!(g.to_string(), c);
        }
    }

    #[test]
    fn fs_relative_strips_mkd_keeps_monkeyking() {
        let g = GroupPath::parse("mkd.projects.mkd.log").unwrap();
        assert_eq!(g.fs_relative(), "projects/mkd/log");
        let m = GroupPath::parse("monkeyking").unwrap();
        assert_eq!(m.fs_relative(), "monkeyking");
        let ml = GroupPath::parse("monkeyking.log").unwrap();
        assert_eq!(ml.fs_relative(), "monkeyking/log");
    }

    #[test]
    fn monkeyking_helpers() {
        assert_eq!(GroupPath::monkeyking().as_str(), "monkeyking");
        assert_eq!(GroupPath::monkeyking_sub("log").as_str(), "monkeyking.log");
        assert_eq!(
            GroupPath::monkeyking_sub("projects.specs").as_str(),
            "monkeyking.projects.specs"
        );
    }

    #[test]
    fn project_tasks_helper() {
        let g = GroupPath::project_tasks(HierarchyBranch::Projects, "mkd");
        assert_eq!(g.as_str(), "mkd.projects.mkd.tasks");
        let a = GroupPath::project_tasks(HierarchyBranch::Agents, "hephaestus");
        assert_eq!(a.as_str(), "mkd.agents.hephaestus.tasks");
        let s = GroupPath::project_tasks(HierarchyBranch::System, "ignored");
        assert_eq!(s.as_str(), "mkd.system.tasks");
    }

    #[test]
    #[allow(deprecated)]
    fn task_group_helper_legacy() {
        let g = GroupPath::task_group(
            HierarchyBranch::Projects,
            "mkd",
            "1780536515909",
            "fix-nntp-drift",
        );
        assert_eq!(
            g.as_str(),
            "mkd.projects.mkd.tasks.1780536515909-fix-nntp-drift"
        );
        let g2 = GroupPath::task_group(HierarchyBranch::Projects, "mkd", "42", "");
        assert_eq!(g2.as_str(), "mkd.projects.mkd.tasks.42");
    }

    #[test]
    fn is_legacy_task_child_group_detects_leaves() {
        assert!(!GroupPath::is_legacy_task_child_group(
            "mkd.projects.mkd.tasks"
        ));
        assert!(!GroupPath::is_legacy_task_child_group("mkd.system.tasks"));
        assert!(GroupPath::is_legacy_task_child_group(
            "mkd.projects.mkd.tasks.42-foo"
        ));
        assert!(GroupPath::is_legacy_task_child_group(
            "mkd.projects.mkd.tasks.task-249"
        ));
        assert!(!GroupPath::is_legacy_task_child_group(
            "mkd.projects.mkd.log"
        ));
    }

    #[test]
    fn sanitize_token_basics() {
        assert_eq!(sanitize_token("Monkey King"), "monkey-king");
        assert_eq!(sanitize_token("  "), "unknown");
    }
}

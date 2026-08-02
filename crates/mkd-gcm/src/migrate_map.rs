// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Migration-only wing/room → group path oracle.
//!
//! **Not** part of the default runtime public API. Enable with Cargo feature
//! `locator-migrate` for one-shot migrators. Mapping is frozen to the
//! pre-cutover `GroupPath::from_legacy_wing_room` algorithm so existing rows
//! rewrite to the same `mkd.*` strings the wire already uses.
//!
//! Runtime request handling MUST use [`GroupPath::parse`] / full group paths only.
//!
//! ## Addressbook
//!
//! The frozen [`classify_wing`] lists agents / users / system only; **no legacy
//! wing token maps to [`HierarchyBranch::Addressbook`]** (unknown → Projects).
//! Addressbook paths still appear on-wire (`mkd.addressbook.*`) and are
//! covered by reverse mapping + the explicit branch helper
//! [`from_legacy_wing_room_for_branch`] so the Addressbook format arm cannot
//! drift silently. Forward migration of historical wing+room pairs never
//! produced addressbook groups via this classifier.

use crate::path::{GroupPath, HierarchyBranch, normalize_task_leaf, sanitize_token};

/// Map legacy wing + room to a group path (exporter-compatible, frozen oracle).
///
/// Branch is chosen by [`classify_wing`] (never Addressbook — see module docs).
pub fn from_legacy_wing_room(wing: &str, room: &str) -> GroupPath {
    from_legacy_wing_room_for_branch(wing, room, classify_wing(wing))
}

/// Same mapping as [`from_legacy_wing_room`] with an **explicit**
/// [`HierarchyBranch`] (bypasses [`classify_wing`]).
///
/// Used to pin the Addressbook / other format arms in tests and for migrators
/// that already know the branch independent of the legacy wing token lists.
pub fn from_legacy_wing_room_for_branch(
    wing: &str,
    room: &str,
    branch: HierarchyBranch,
) -> GroupPath {
    let w = sanitize_token(wing);
    let mut r = sanitize_token(room);
    if r.is_empty() || r == "." || r == "general" || r == "overview" {
        r = "main".into();
    }

    let base = match branch {
        HierarchyBranch::System => "mkd.system".to_string(),
        HierarchyBranch::Users => format!("mkd.users.{w}"),
        HierarchyBranch::Agents => {
            let agent = if w == "monkey-king" {
                "monkeyking".to_string()
            } else {
                w.clone()
            };
            format!("mkd.agents.{agent}")
        }
        HierarchyBranch::Addressbook => format!("mkd.addressbook.{w}"),
        HierarchyBranch::Projects => format!("mkd.projects.{w}"),
    };

    if r == "tasks" {
        return GroupPath::unchecked(format!("{base}.tasks"));
    }
    if let Some(task_leaf) = normalize_task_leaf(room, None) {
        return GroupPath::unchecked(format!("{base}.tasks.{task_leaf}"));
    }

    if matches!(branch, HierarchyBranch::System) {
        return GroupPath::unchecked(format!("mkd.system.{r}"));
    }
    GroupPath::unchecked(format!("{base}.{r}"))
}

/// Best-effort reverse map for migration diagnostics (not a product dual-read path).
pub fn to_legacy_wing_room(path: &GroupPath) -> Option<(String, String)> {
    let rest = path.as_str().strip_prefix("mkd.")?;
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    match parts[0] {
        "system" => {
            let room = if parts.len() == 1 {
                "main".into()
            } else {
                parts[1..].join(".")
            };
            Some(("system".into(), room))
        }
        "projects" | "agents" | "users" | "addressbook" if parts.len() >= 2 => {
            let wing = parts[1].to_string();
            let room = if parts.len() == 2 {
                "main".into()
            } else if parts.len() >= 4 && parts[2] == "tasks" {
                // …tasks.<id>-<slug> → room <id>-<slug>
                parts[3..].join(".")
            } else {
                parts[2..].join(".")
            };
            Some((wing, room))
        }
        _ => None,
    }
}

/// Classify a legacy wing token into a hierarchy branch (frozen lists).
///
/// **Never returns [`HierarchyBranch::Addressbook`]** — pre-cutover oracle had
/// no addressbook wing-name list; unknown tokens → Projects. See module docs.
pub fn classify_wing(wing: &str) -> HierarchyBranch {
    let w = sanitize_token(wing);
    const AGENTS: &[&str] = &[
        "monkey-king",
        "monkeyking",
        "grok",
        "grok-build",
        "architecture",
        "code",
        "devops",
        "hephaestus",
        "thesmith",
        "sherlock",
        "daedalus",
        "patton",
        "minerva",
        "janus",
        "argus",
    ];
    const USERS: &[&str] = &["nate"];
    const SYSTEM: &[&str] = &["system", "system-tasks", "system_tasks"];
    if SYSTEM.contains(&wing) || SYSTEM.contains(&w.as_str()) || w == "system" {
        return HierarchyBranch::System;
    }
    if USERS.contains(&wing) || USERS.contains(&w.as_str()) {
        return HierarchyBranch::Users;
    }
    if AGENTS.contains(&wing) || AGENTS.contains(&w.as_str()) {
        return HierarchyBranch::Agents;
    }
    HierarchyBranch::Projects
}

/// Sub-hierarchy base path for a legacy wing name (no leaf room).
///
/// Uses `from_legacy_wing_room(wing, "main")` and strips the trailing `.main`
/// segment so association tables store e.g. `mkd.projects.mkd`.
pub fn migrate_wing_base(wing: &str) -> GroupPath {
    let full = from_legacy_wing_room(wing, "main");
    let s = full.as_str();
    if let Some(base) = s.strip_suffix(".main") {
        GroupPath::unchecked(base.to_string())
    } else {
        full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- golden matrix: projects, agents, users, system, addressbook, task leaf ---

    #[test]
    fn golden_projects() {
        let g = from_legacy_wing_room("mkd", "log");
        assert_eq!(g.as_str(), "mkd.projects.mkd.log");
        assert_eq!(g.fs_relative(), "projects/mkd/log");
    }

    #[test]
    fn golden_agents() {
        let g = from_legacy_wing_room("monkey-king", "log");
        assert_eq!(g.as_str(), "mkd.agents.monkeyking.log");
        let g2 = from_legacy_wing_room("grok", "conversations");
        assert_eq!(g2.as_str(), "mkd.agents.grok.conversations");
    }

    #[test]
    fn golden_users() {
        assert_eq!(
            from_legacy_wing_room("nate", "log").as_str(),
            "mkd.users.nate.log"
        );
    }

    #[test]
    fn golden_system() {
        assert_eq!(
            from_legacy_wing_room("system", "rehearsal").as_str(),
            "mkd.system.rehearsal"
        );
    }

    #[test]
    fn golden_addressbook_forward_via_explicit_branch() {
        // Exercise the Addressbook arm of the mapper (format pin).
        // classify_wing never yields Addressbook (frozen); use explicit branch.
        assert_ne!(
            classify_wing("contacts"),
            HierarchyBranch::Addressbook,
            "frozen classify_wing must not invent addressbook from wing tokens"
        );
        let g = from_legacy_wing_room_for_branch("contacts", "main", HierarchyBranch::Addressbook);
        assert_eq!(g.as_str(), "mkd.addressbook.contacts.main");
        let g2 =
            from_legacy_wing_room_for_branch("contacts", "entries", HierarchyBranch::Addressbook);
        assert_eq!(g2.as_str(), "mkd.addressbook.contacts.entries");
    }

    #[test]
    fn golden_addressbook_reverse() {
        let g = GroupPath::parse("mkd.addressbook.contacts.main").unwrap();
        assert_eq!(
            to_legacy_wing_room(&g),
            Some(("contacts".into(), "main".into()))
        );
    }

    #[test]
    fn classify_wing_never_returns_addressbook() {
        for token in [
            "mkd",
            "contacts",
            "addressbook",
            "nate",
            "system",
            "grok",
            "unknown-thing",
        ] {
            assert_ne!(
                classify_wing(token),
                HierarchyBranch::Addressbook,
                "classify_wing({token}) unexpectedly Addressbook"
            );
        }
    }

    #[test]
    fn golden_task_leaf() {
        // Legacy `task-<id>` → bare id under …tasks.
        let g = from_legacy_wing_room("mkd", "task-1780536515909");
        assert_eq!(g.as_str(), "mkd.projects.mkd.tasks.1780536515909");
        // `merge-<id>` alias still recognized.
        let g2 = from_legacy_wing_room("mkd", "merge-42");
        assert_eq!(g2.as_str(), "mkd.projects.mkd.tasks.42");
        // Legacy `hall-…` no longer special; falls through to leaf room name.
        let g3 = from_legacy_wing_room("mkd", "hall-task-99");
        assert_eq!(g3.as_str(), "mkd.projects.mkd.hall-task-99");
        // Slugged task leaf form (product construction via task_group):
        let g4 = GroupPath::task_group(
            HierarchyBranch::Projects,
            "mkd",
            "1780536515909",
            "fix-nntp-drift",
        );
        assert_eq!(
            g4.as_str(),
            "mkd.projects.mkd.tasks.1780536515909-fix-nntp-drift"
        );
    }

    #[test]
    fn reverse_wing_room() {
        let g = GroupPath::parse("mkd.projects.mkd.log").unwrap();
        assert_eq!(to_legacy_wing_room(&g), Some(("mkd".into(), "log".into())));
        let t = GroupPath::parse("mkd.projects.mkd.tasks.1780536515909-fix-nntp-drift").unwrap();
        assert_eq!(
            to_legacy_wing_room(&t),
            Some(("mkd".into(), "1780536515909-fix-nntp-drift".into()))
        );
    }

    #[test]
    fn migrate_wing_base_strips_main() {
        assert_eq!(migrate_wing_base("mkd").as_str(), "mkd.projects.mkd");
        assert_eq!(migrate_wing_base("nate").as_str(), "mkd.users.nate");
    }

    #[test]
    fn general_and_overview_room_become_main() {
        // Empty string sanitizes to "unknown" (frozen sanitize_token), not main.
        assert_eq!(
            from_legacy_wing_room("mkd", "").as_str(),
            "mkd.projects.mkd.unknown"
        );
        assert_eq!(
            from_legacy_wing_room("mkd", "general").as_str(),
            "mkd.projects.mkd.main"
        );
        assert_eq!(
            from_legacy_wing_room("mkd", "overview").as_str(),
            "mkd.projects.mkd.main"
        );
    }
}

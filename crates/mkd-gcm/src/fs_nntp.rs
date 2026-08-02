// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Filesystem NNTP layout store — bulk import / ensure_group / disaster recovery.
//! **Not** the default agent write path (bypasses server redaction).

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

use crate::MemoryStore;
use crate::path::GroupPath;
use crate::types::{Article, GroupInfo, PostHeaders, PostRef};

/// Write articles under `MKD_NNTP_ROOT` matching mkd-gcm-server layout.
#[derive(Debug, Clone)]
pub struct FsNntpMemoryStore {
    root: PathBuf,
    operator_fqdn: String,
}

impl FsNntpMemoryStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            operator_fqdn: std::env::var("MKD_NNTP_OPERATOR_FQDN")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "localhost".into()),
        }
    }

    pub fn with_operator_fqdn(mut self, fqdn: impl Into<String>) -> Self {
        self.operator_fqdn = fqdn.into();
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn group_dir(&self, group: &GroupPath) -> PathBuf {
        self.root.join(group.fs_relative())
    }
}

impl MemoryStore for FsNntpMemoryStore {
    fn ensure_group(&self, path: &GroupPath) -> Result<()> {
        let dir = self.group_dir(path);
        std::fs::create_dir_all(&dir).map_err(Error::Io)?;
        Ok(())
    }

    fn post(&self, group: &GroupPath, headers: &PostHeaders, body: &str) -> Result<PostRef> {
        self.ensure_group(group)?;
        let slot = headers
            .get("X-Mkd-Slot")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "note".into());
        let slot_dir = self.group_dir(group).join(&slot);
        std::fs::create_dir_all(&slot_dir).map_err(Error::Io)?;
        let version = next_version(&slot_dir)?;
        let message_id = headers
            .get("Message-ID")
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("<{slot}-v{version}@{}>", self.operator_fqdn));
        let subject = headers.get("Subject").unwrap_or("mkd post");
        let from = headers
            .get("From")
            .unwrap_or("mkd-agent <mkd-agent@localhost>");
        let date = headers
            .get("Date")
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc2822());

        let mut article = String::new();
        article.push_str(&format!("Newsgroups: {}\n", group.as_str()));
        article.push_str(&format!("Message-ID: {message_id}\n"));
        article.push_str(&format!("Subject: {subject}\n"));
        article.push_str(&format!("From: {from}\n"));
        article.push_str(&format!("Date: {date}\n"));
        article.push_str(&format!("X-Mkd-Loc: {}\n", group.as_str()));
        article.push_str(&format!("X-Mkd-Slot: {slot}\n"));
        article.push_str(&format!("X-Mkd-Version: {version}\n"));
        for (k, v) in &headers.fields {
            if k.eq_ignore_ascii_case("Newsgroups")
                || k.eq_ignore_ascii_case("Message-ID")
                || k.eq_ignore_ascii_case("Subject")
                || k.eq_ignore_ascii_case("From")
                || k.eq_ignore_ascii_case("Date")
                || k.eq_ignore_ascii_case("X-Mkd-Loc")
                || k.eq_ignore_ascii_case("X-Mkd-Slot")
                || k.eq_ignore_ascii_case("X-Mkd-Version")
            {
                continue;
            }
            article.push_str(&format!("{k}: {v}\n"));
        }
        article.push('\n');
        article.push_str(body);
        if !body.ends_with('\n') {
            article.push('\n');
        }

        let path = slot_dir.join(format!("{version}.md"));
        std::fs::write(&path, article.as_bytes()).map_err(Error::Io)?;
        Ok(PostRef {
            message_id,
            group: group.clone(),
            slot: Some(slot),
            version: Some(version),
        })
    }

    fn get_article(&self, post: &PostRef) -> Result<Article> {
        let slot = post
            .slot
            .as_deref()
            .ok_or_else(|| Error::Message("FsNntp get requires slot".into()))?;
        let version = post
            .version
            .ok_or_else(|| Error::Message("FsNntp get requires version".into()))?;
        let path = self
            .group_dir(&post.group)
            .join(slot)
            .join(format!("{version}.md"));
        let raw = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound(path.display().to_string())
            } else {
                Error::Io(e)
            }
        })?;
        let (hdr_part, body) = raw.split_once("\n\n").unwrap_or((&raw, ""));
        let mut headers = std::collections::BTreeMap::new();
        for line in hdr_part.lines() {
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Ok(Article {
            post_ref: post.clone(),
            headers,
            body: body.to_string(),
        })
    }

    fn list_groups(&self, wildmat: Option<&str>) -> Result<Vec<GroupInfo>> {
        // Full tree walk is expensive; return empty until a cheap listing exists.
        let _ = wildmat;
        Ok(Vec::new())
    }

    fn backend_name(&self) -> &'static str {
        "fs"
    }
}

fn next_version(slot_dir: &Path) -> Result<u32> {
    let mut max = 0u32;
    if slot_dir.is_dir() {
        for ent in std::fs::read_dir(slot_dir).map_err(Error::Io)? {
            let ent = ent.map_err(Error::Io)?;
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".md")
                && let Ok(n) = stem.parse::<u32>()
            {
                max = max.max(n);
            }
        }
    }
    Ok(max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_and_post_roundtrip() {
        let dir = tempdir().unwrap();
        let store = FsNntpMemoryStore::new(dir.path()).with_operator_fqdn("lab.example");
        let g = GroupPath::parse("mkd.projects.mkd.log").unwrap();
        store.ensure_group(&g).unwrap();
        assert!(dir.path().join("projects/mkd/log").is_dir());
        let pref = store
            .post(
                &g,
                &PostHeaders::new()
                    .with("Subject", "hello")
                    .with("X-Mkd-Slot", "t1"),
                "body text",
            )
            .unwrap();
        assert!(pref.message_id.contains("@lab.example"));
        let art = store.get_article(&pref).unwrap();
        assert!(art.body.contains("body text"));
    }
}

// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Shared GCM memory article and overview types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::path::GroupPath;

/// Headers for a POST (case-preserving keys; Newsgroups filled from group if absent).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostHeaders {
    /// Ordered header map (Subject, From, X-Mkd-Slot, …).
    pub fields: BTreeMap<String, String>,
}

impl PostHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(name.into(), value.into());
        self
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(name.into(), value.into());
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Wire header block (no trailing blank line).
    pub fn to_wire(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.fields {
            out.push_str(k);
            out.push_str(": ");
            out.push_str(v);
            out.push_str("\r\n");
        }
        out
    }
}

/// Reference to a stored post after write or lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostRef {
    pub message_id: String,
    pub group: GroupPath,
    pub slot: Option<String>,
    pub version: Option<u32>,
}

/// Full article payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub post_ref: PostRef,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

/// LIST ACTIVE-style group summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub high: u64,
    pub low: u64,
    pub status: char,
}

/// XOVER row (subset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewRow {
    pub number: u64,
    pub subject: String,
    pub from: String,
    pub date: String,
    pub message_id: String,
    pub bytes: u64,
    pub lines: u64,
}

/// Search hit from a backend that implements full-text search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub message_id: String,
    pub group: String,
    pub subject: String,
    pub snippet: String,
}

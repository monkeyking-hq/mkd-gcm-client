// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Lightweight errors for the GCM client (no SQLite / pool / core deps).

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("gcm error: {0}")]
    Message(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

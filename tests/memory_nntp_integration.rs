// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! Live freeport proof: NntpMemoryStore POST → ARTICLE roundtrip.
//!
//! Requires a built `mkd-gcm-server` binary (legacy name: `mkd-nntp-server`).
//! Discovers via:
//! 1. `MKD_NNTP_SERVER_BIN` / `MKD_GCM_SERVER_BIN`
//! 2. sibling `../mkd-gcm-server|mkd-nntp-server/target/{release,debug}/mkd-gcm-server|mkd-nntp-server`
//! 3. `mkd-gcm-server` / `mkd-nntp-server` on PATH
//!
//! Always allocates a free port at runtime (never hardcodes 1119).
//!
//! Run: `cargo test -p mkd-gcm --test memory_nntp_integration -- --nocapture`

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use mkd_gcm::{GroupPath, MemoryStore, NntpMemoryStore, PostHeaders};
use tempfile::tempdir;

fn freeport() -> u16 {
    // Optional freeport helper on PATH; otherwise bind ephemeral.
    if let Ok(out) = Command::new("mkd-freeport").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout);
        if let Ok(p) = s.trim().parse::<u16>() {
            return p;
        }
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral bind");
    listener.local_addr().unwrap().port()
}

fn find_nntp_bin() -> Option<PathBuf> {
    for env_key in ["MKD_GCM_SERVER_BIN", "MKD_NNTP_SERVER_BIN"] {
        if let Ok(p) = std::env::var(env_key) {
            let pb = PathBuf::from(p);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Prefer renamed paths; keep legacy checkout/binary names during transition.
    for rel in [
        "../mkd-gcm-server/target/release/mkd-gcm-server",
        "../mkd-gcm-server/target/debug/mkd-gcm-server",
        "../mkd-nntp-server/target/release/mkd-gcm-server",
        "../mkd-nntp-server/target/debug/mkd-gcm-server",
        "../mkd-nntp-server/target/release/mkd-nntp-server",
        "../mkd-nntp-server/target/debug/mkd-nntp-server",
        "../mkd-gcm-server/target/release/mkd-nntp-server",
        "../mkd-gcm-server/target/debug/mkd-nntp-server",
    ] {
        let cand = manifest.join(rel);
        if cand.is_file() {
            return Some(cand);
        }
    }
    which_bin("mkd-gcm-server").or_else(|| which_bin("mkd-nntp-server"))
}

fn which_bin(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

struct ServerGuard {
    child: Child,
    #[allow(dead_code)]
    root: tempfile::TempDir,
    port: u16,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_server() -> Option<ServerGuard> {
    let bin = find_nntp_bin()?;
    let root = tempdir().ok()?;
    let port = freeport();
    let child = Command::new(&bin)
        .env("MKD_NNTP_ROOT", root.path())
        .env("MKD_NNTP_PORT", port.to_string())
        .env("MKD_NNTP_OPERATOR_FQDN", "lab.example.internal")
        .env("MKD_NNTP_TLS", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Wait for accept
    for _ in 0..50 {
        if TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(100),
        )
        .is_ok()
        {
            return Some(ServerGuard { child, root, port });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let mut guard = ServerGuard { child, root, port };
    let _ = guard.child.kill();
    None
}

#[test]
fn nntp_memory_store_post_creates_article_freeport() {
    let Some(server) = spawn_server() else {
        eprintln!(
            "SKIP: mkd-gcm-server binary not found or failed to start (set MKD_GCM_SERVER_BIN or MKD_NNTP_SERVER_BIN)"
        );
        return;
    };
    let store = NntpMemoryStore::new("127.0.0.1", server.port)
        .with_default_from("agent <agent@lab.example.internal>");
    let group = GroupPath::parse("mkd.projects.gcm-client-it.log").expect("group path");
    store.ensure_group(&group).expect("ensure");
    let body = format!(
        "GCM client freeport integration proof freeport={} ts={}",
        server.port,
        chrono::Utc::now().to_rfc3339()
    );
    let pref = store
        .post(
            &group,
            &PostHeaders::new()
                .with("Subject", "gcm-client freeport proof")
                .with("X-Mkd-Slot", "freeport-proof"),
            &body,
        )
        .expect("POST via NntpMemoryStore");
    assert!(
        pref.message_id.contains('@'),
        "expected FQDN Message-ID, got {}",
        pref.message_id
    );
    assert_eq!(pref.group.as_str(), "mkd.projects.gcm-client-it.log");

    // ARTICLE roundtrip via store
    let art = store.get_article(&pref).expect("ARTICLE");
    assert!(
        art.body.contains("GCM client freeport") || art.body.contains("integration proof"),
        "body missing proof text: {}",
        art.body
    );

    // Independent raw TCP check (not just trusting the client library)
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.starts_with("200 "));
    write!(stream, "GROUP {}\r\n", group.as_str()).unwrap();
    line.clear();
    reader.read_line(&mut line).unwrap();
    assert!(line.starts_with("211 "), "GROUP failed: {line}");
    write!(stream, "ARTICLE {}\r\n", pref.message_id).unwrap();
    line.clear();
    reader.read_line(&mut line).unwrap();
    assert!(
        line.starts_with("220 "),
        "raw ARTICLE failed: {line} (msgid={})",
        pref.message_id
    );
    eprintln!(
        "EVIDENCE: NntpMemoryStore POST ok port={} msgid={} group={}",
        server.port,
        pref.message_id,
        group.as_str()
    );
}

// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! NNTP-backed [`MemoryStore`] — agent write path (POST) preferred.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::error::{Error, Result};

use crate::MemoryStore;
use crate::path::GroupPath;
use crate::types::{Article, GroupInfo, OverviewRow, PostHeaders, PostRef};

/// Blocking NNTP client implementing [`MemoryStore`].
#[derive(Debug, Clone)]
pub struct NntpMemoryStore {
    host: String,
    port: u16,
    /// AUTHINFO USER (optional).
    user: Option<String>,
    /// AUTHINFO PASS (optional).
    pass: Option<String>,
    /// Named PAT / session bearer for AUTHINFO SASL OAUTHBEARER.
    bearer_token: Option<String>,
    /// Optional authzid for OAUTHBEARER GS2 header (e.g. email).
    bearer_authzid: Option<String>,
    /// Default From: mailbox if headers omit it.
    default_from: String,
    timeout: Duration,
}

impl NntpMemoryStore {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            user: None,
            pass: None,
            bearer_token: None,
            bearer_authzid: None,
            default_from: "mkd-agent <mkd-agent@localhost>".into(),
            timeout: Duration::from_secs(15),
        }
    }

    /// Parse `host:port`, `nntp://host:port`, or bare host (port 1119).
    ///
    /// Host may be a DNS name (e.g. Compose service `mkd-gcm-server`) or an IP.
    /// Resolution happens at connect time via [`ToSocketAddrs`].
    pub fn from_url(url: &str) -> Result<Self> {
        let s = url.trim();
        let s = s
            .strip_prefix("nntp://")
            .or_else(|| s.strip_prefix("NNTP://"))
            .unwrap_or(s);
        // Strip path/query if present (nntp://host:port/extra)
        let s = s.split('/').next().unwrap_or(s);
        let (host, port) = if let Some((h, p)) = s.rsplit_once(':') {
            // Avoid treating IPv6 literals without brackets as host:port incorrectly;
            // lab/compose URLs are hostnames or IPv4.
            if p.chars().all(|c| c.is_ascii_digit()) {
                let port: u16 = p
                    .parse()
                    .map_err(|_| Error::Config(format!("bad NNTP port in {url}")))?;
                (h.to_string(), port)
            } else {
                (s.to_string(), 1119)
            }
        } else {
            (s.to_string(), 1119)
        };
        if host.is_empty() {
            return Err(Error::Config("empty NNTP host".into()));
        }
        Ok(Self::new(host, port))
    }

    pub fn with_auth(mut self, user: impl Into<String>, pass: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self.pass = Some(pass.into());
        self
    }

    /// Authenticate with a membership session token or named PAT (OAUTHBEARER).
    ///
    /// Server may require TLS before SASL OAUTHBEARER. Prefer this for
    /// project-submit tokens issued by your GCM operator surface.
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    /// Optional authzid (e.g. submitter or service email) for OAUTHBEARER GS2.
    pub fn with_bearer_authzid(mut self, authzid: impl Into<String>) -> Self {
        self.bearer_authzid = Some(authzid.into());
        self
    }

    pub fn with_default_from(mut self, from: impl Into<String>) -> Self {
        self.default_from = from.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn connect(&self) -> Result<NntpSession> {
        // Resolve hostname (e.g. service DNS `mkd-gcm-server:1119`) — do not
        // require a pre-parsed SocketAddr (IP-only).
        let addr = format!("{}:{}", self.host, self.port);
        use std::net::ToSocketAddrs;
        let mut addrs = addr
            .to_socket_addrs()
            .map_err(|e| Error::Config(format!("NNTP resolve {addr}: {e}")))?;
        let sock = addrs
            .next()
            .ok_or_else(|| Error::Config(format!("NNTP resolve {addr}: no addresses")))?;
        let stream = TcpStream::connect_timeout(&sock, self.timeout)
            .map_err(|e| Error::Message(format!("NNTP connect {addr} ({sock}): {e}")))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(Error::Io)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(Error::Io)?;
        let mut sess = NntpSession {
            reader: BufReader::new(stream),
        };
        let greet = sess.read_line()?;
        if !greet.starts_with("200 ") && !greet.starts_with("201 ") {
            return Err(Error::Message(format!("NNTP greeting: {greet}")));
        }
        // MODE READER — posting allowed when primary
        sess.cmd("MODE READER")?;
        let mode = sess.read_line()?;
        if !(mode.starts_with("200 ") || mode.starts_with("201 ")) {
            return Err(Error::Message(format!("MODE READER: {mode}")));
        }
        // Prefer OAUTHBEARER (named PAT / session) when set.
        if let Some(token) = &self.bearer_token {
            let b64 = encode_oauthbearer_message(self.bearer_authzid.as_deref(), token);
            sess.cmd(&format!("AUTHINFO SASL OAUTHBEARER {b64}"))?;
            let r = sess.read_line()?;
            if !r.starts_with("281 ") {
                return Err(Error::Message(format!("AUTHINFO SASL OAUTHBEARER: {r}")));
            }
        } else if let (Some(u), Some(p)) = (&self.user, &self.pass) {
            sess.cmd(&format!("AUTHINFO USER {u}"))?;
            let r = sess.read_line()?;
            if r.starts_with("381 ") {
                sess.cmd(&format!("AUTHINFO PASS {p}"))?;
                let r2 = sess.read_line()?;
                if !r2.starts_with("281 ") {
                    return Err(Error::Message(format!("AUTHINFO PASS: {r2}")));
                }
            } else if !r.starts_with("281 ") {
                return Err(Error::Message(format!("AUTHINFO USER: {r}")));
            }
        }
        Ok(sess)
    }

    /// POST a JSON body (`Content-Type: application/json`) to `group`.
    ///
    /// Used by language-correction and other structured host pipelines.
    pub fn post_json(
        &self,
        group: &GroupPath,
        subject: &str,
        body_json: &str,
        extra: &PostHeaders,
    ) -> Result<PostRef> {
        let mut headers = extra.clone();
        if headers.get("Subject").is_none() {
            headers.set("Subject", subject);
        }
        if headers.get("Content-Type").is_none() {
            headers.set("Content-Type", "application/json; charset=utf-8");
        }
        if headers.get("X-Mkd-Format").is_none() {
            headers.set("X-Mkd-Format", "json");
        }
        self.post(group, &headers, body_json)
    }
}

/// RFC 7628 one-shot OAUTHBEARER initial response (matches mkd-gcm-server).
pub fn encode_oauthbearer_message(authzid: Option<&str>, token: &str) -> String {
    use base64::Engine;
    let gs2 = match authzid {
        Some(a) if !a.is_empty() => format!("n,a={a},"),
        _ => "n,".to_string(),
    };
    let raw = format!("{gs2}\u{0001}auth=Bearer {token}\u{0001}\u{0001}");
    base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
}

struct NntpSession {
    reader: BufReader<TcpStream>,
}

impl NntpSession {
    fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).map_err(Error::Io)?;
        if n == 0 {
            return Err(Error::Message("NNTP EOF".into()));
        }
        Ok(line.trim_end_matches(['\r', '\n']).to_string())
    }

    fn cmd(&mut self, line: &str) -> Result<()> {
        let stream = self.reader.get_mut();
        stream
            .write_all(format!("{line}\r\n").as_bytes())
            .map_err(Error::Io)?;
        stream.flush().map_err(Error::Io)
    }
}

impl MemoryStore for NntpMemoryStore {
    fn ensure_group(&self, _path: &GroupPath) -> Result<()> {
        // Lab MVP: server accepts any mkd.* and creates dirs on POST.
        // Multi-user: Control newgroup under auth (future).
        Ok(())
    }

    fn post(&self, group: &GroupPath, headers: &PostHeaders, body: &str) -> Result<PostRef> {
        let mut sess = self.connect()?;
        sess.cmd("POST")?;
        let ready = sess.read_line()?;
        if !ready.starts_with("340 ") {
            return Err(Error::Message(format!("POST not accepted: {ready}")));
        }

        let mut hdrs = headers.clone();
        if hdrs.get("Newsgroups").is_none() {
            hdrs.set("Newsgroups", group.as_str());
        }
        if hdrs.get("Subject").is_none() {
            let subj = body.lines().next().unwrap_or("mkd post");
            let subj = if subj.len() > 72 { &subj[..72] } else { subj };
            hdrs.set("Subject", subj);
        }
        if hdrs.get("From").is_none() {
            hdrs.set("From", &self.default_from);
        }
        if hdrs.get("X-Mkd-Slot").is_none() {
            let slot = derive_slot(hdrs.get("Subject").unwrap_or("note"));
            hdrs.set("X-Mkd-Slot", slot);
        }

        let mut article = hdrs.to_wire();
        article.push_str("\r\n");
        // Body: normalize newlines; dot-stuff lines starting with .
        for line in body.split('\n') {
            let line = line.trim_end_matches('\r');
            if line.starts_with('.') {
                article.push('.');
            }
            article.push_str(line);
            article.push_str("\r\n");
        }
        article.push_str(".\r\n");

        {
            let stream = sess.reader.get_mut();
            stream.write_all(article.as_bytes()).map_err(Error::Io)?;
            stream.flush().map_err(Error::Io)?;
        }
        let resp = sess.read_line()?;
        if !resp.starts_with("240 ") {
            return Err(Error::Message(format!("POST rejected: {resp}")));
        }
        // "240 Article posted <msgid>" or "240 article posted ok"
        let message_id = extract_msgid_from_240(&resp).unwrap_or_else(|| {
            hdrs.get("Message-ID")
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("<unknown@{}>", group.as_str()))
        });
        let slot = hdrs.get("X-Mkd-Slot").map(|s| s.to_string());
        let _ = sess.cmd("QUIT");
        let _ = sess.read_line();
        Ok(PostRef {
            message_id,
            group: group.clone(),
            slot,
            version: None,
        })
    }

    fn get_article(&self, post: &PostRef) -> Result<Article> {
        let mut sess = self.connect()?;
        let mid = if post.message_id.starts_with('<') {
            post.message_id.clone()
        } else {
            format!("<{}>", post.message_id)
        };
        // Some servers (mkd-gcm-server) require GROUP selected before ARTICLE
        // even when Message-ID is absolute — select first when we know the group.
        if !post.group.as_str().is_empty() {
            sess.cmd(&format!("GROUP {}", post.group.as_str()))?;
            let g = sess.read_line()?;
            if !g.starts_with("211 ") {
                return Err(Error::NotFound(format!(
                    "GROUP {}: {g}",
                    post.group.as_str()
                )));
            }
        }
        sess.cmd(&format!("ARTICLE {mid}"))?;
        let head = sess.read_line()?;
        if !head.starts_with("220 ") {
            return Err(Error::NotFound(format!("ARTICLE {mid}: {head}")));
        }
        let mut headers = std::collections::BTreeMap::new();
        let mut body = String::new();
        let mut in_body = false;
        loop {
            let line = sess.read_line()?;
            if line == "." {
                break;
            }
            if !in_body {
                if line.is_empty() {
                    in_body = true;
                    continue;
                }
                if let Some((k, v)) = line.split_once(':') {
                    headers.insert(k.trim().to_string(), v.trim().to_string());
                }
            } else {
                let real = line.strip_prefix('.').unwrap_or(&line);
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(real);
            }
        }
        let slot = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("X-Mkd-Slot"))
            .map(|(_, v)| v.clone())
            .or_else(|| post.slot.clone());
        let version = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("X-Mkd-Version"))
            .and_then(|(_, v)| v.parse().ok())
            .or(post.version);
        let group = headers
            .iter()
            .find(|(k, _)| {
                k.eq_ignore_ascii_case("Newsgroups") || k.eq_ignore_ascii_case("X-Mkd-Loc")
            })
            .and_then(|(_, v)| GroupPath::parse(v.split(',').next().unwrap_or(v).trim()).ok())
            .unwrap_or_else(|| post.group.clone());
        let message_id = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("Message-ID"))
            .map(|(_, v)| v.clone())
            .unwrap_or(mid);
        let _ = sess.cmd("QUIT");
        let _ = sess.read_line();
        Ok(Article {
            post_ref: PostRef {
                message_id,
                group,
                slot,
                version,
            },
            headers,
            body,
        })
    }

    fn list_groups(&self, wildmat: Option<&str>) -> Result<Vec<GroupInfo>> {
        let mut sess = self.connect()?;
        if let Some(w) = wildmat {
            sess.cmd(&format!("LIST ACTIVE {w}"))?;
        } else {
            sess.cmd("LIST ACTIVE")?;
        }
        let head = sess.read_line()?;
        if !head.starts_with("215 ") {
            return Err(Error::Message(format!("LIST ACTIVE: {head}")));
        }
        let mut out = Vec::new();
        loop {
            let line = sess.read_line()?;
            if line == "." {
                break;
            }
            // group high low status
            let mut parts = line.split_whitespace();
            if let (Some(name), Some(high), Some(low), Some(st)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            {
                out.push(GroupInfo {
                    name: name.to_string(),
                    high: high.parse().unwrap_or(0),
                    low: low.parse().unwrap_or(0),
                    status: st.chars().next().unwrap_or('y'),
                });
            }
        }
        let _ = sess.cmd("QUIT");
        let _ = sess.read_line();
        Ok(out)
    }

    fn over(&self, group: &GroupPath, range: Option<&str>) -> Result<Vec<OverviewRow>> {
        let mut sess = self.connect()?;
        sess.cmd(&format!("GROUP {}", group.as_str()))?;
        let g = sess.read_line()?;
        if !g.starts_with("211 ") {
            return Err(Error::NotFound(format!("GROUP {}: {g}", group.as_str())));
        }
        match range {
            Some(r) if !r.trim().is_empty() => sess.cmd(&format!("XOVER {}", r.trim()))?,
            _ => sess.cmd("XOVER")?,
        }
        let head = sess.read_line()?;
        if head.starts_with("420 ") {
            let _ = sess.cmd("QUIT");
            let _ = sess.read_line();
            return Ok(Vec::new());
        }
        if !head.starts_with("224 ") {
            return Err(Error::Message(format!("XOVER: {head}")));
        }
        let mut out = Vec::new();
        loop {
            let line = sess.read_line()?;
            if line == "." {
                break;
            }
            if let Some(row) = parse_xover_line(&line) {
                out.push(row);
            }
        }
        let _ = sess.cmd("QUIT");
        let _ = sess.read_line();
        Ok(out)
    }

    fn backend_name(&self) -> &'static str {
        "nntp"
    }
}

impl NntpMemoryStore {
    /// Resolve a document body from NNTP by group + source_file (Subject / X-Mkd-Source-File).
    ///
    /// Resolve via XOVER (Message-ID inventory per group) then ARTICLE.
    /// Picks highest `X-Mkd-Version` unless `want_version` is set.
    pub fn resolve_document_by_source(
        &self,
        group: &GroupPath,
        source_file: &str,
        want_version: Option<u32>,
    ) -> Result<Article> {
        let rows = self.over(group, None)?;
        if rows.is_empty() {
            return Err(Error::NotFound(format!(
                "no overview rows in {}",
                group.as_str()
            )));
        }
        let base = std::path::Path::new(source_file)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(source_file);
        let mut candidates: Vec<PostRef> = Vec::new();
        for row in &rows {
            let subj = row.subject.trim();
            if subj == source_file || subj == base || subj.ends_with(source_file) {
                candidates.push(PostRef {
                    message_id: row.message_id.clone(),
                    group: group.clone(),
                    slot: None,
                    version: None,
                });
            }
        }
        if candidates.is_empty() {
            // Subject miss: still probe every Message-ID (small lab groups) for source header.
            for row in &rows {
                candidates.push(PostRef {
                    message_id: row.message_id.clone(),
                    group: group.clone(),
                    slot: None,
                    version: None,
                });
            }
        }

        let mut best: Option<Article> = None;
        for pref in candidates {
            let art = match self.get_article(&pref) {
                Ok(a) => a,
                Err(_) => continue,
            };
            let src = art
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("X-Mkd-Source-File"))
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            let subj = art
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("Subject"))
                .map(|(_, v)| v.as_str())
                .unwrap_or("");
            let matches_source = src == source_file
                || src == base
                || subj == source_file
                || subj == base
                || (!src.is_empty() && (src.ends_with(source_file) || src.ends_with(base)));
            if !matches_source {
                continue;
            }
            // Skip pointer stubs when looking for document bodies (caller may pass pointer paths).
            let ver = art
                .post_ref
                .version
                .or_else(|| {
                    art.headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("X-Mkd-Version"))
                        .and_then(|(_, v)| v.parse().ok())
                })
                .unwrap_or(1);
            if let Some(want) = want_version {
                if ver == want {
                    return Ok(art);
                }
                continue;
            }
            let better = match &best {
                None => true,
                Some(prev) => {
                    let prev_v = prev.post_ref.version.unwrap_or(1);
                    ver >= prev_v
                }
            };
            if better {
                let mut art = art;
                art.post_ref.version = Some(ver);
                best = Some(art);
            }
        }
        best.ok_or_else(|| {
            Error::NotFound(format!(
                "no NNTP article for source_file={source_file} in {}",
                group.as_str()
            ))
        })
    }
}

fn extract_msgid_from_240(resp: &str) -> Option<String> {
    // 240 Article posted <id@host>
    let start = resp.find('<')?;
    let end = resp[start..].find('>')? + start;
    Some(resp[start..=end].to_string())
}

/// Parse a single XOVER overview line (tab-separated).
///
/// Classic layout: `num\tSubject\tFrom\tDate\tMessage-ID\tReferences\tBytes\tLines\t…`
fn parse_xover_line(line: &str) -> Option<OverviewRow> {
    let line = line.trim_end_matches(['\r', '\n']);
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 5 {
        return None;
    }
    let number: u64 = parts[0].trim().parse().ok()?;
    let subject = parts[1].to_string();
    let from = parts[2].to_string();
    let date = parts.get(3).unwrap_or(&"").to_string();
    let message_id = parts[4].to_string();
    let bytes = parts
        .get(6)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let lines = parts
        .get(7)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    Some(OverviewRow {
        number,
        subject,
        from,
        date,
        message_id,
        bytes,
        lines,
    })
}

fn derive_slot(subject: &str) -> String {
    let s = subject.trim().to_lowercase();
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if (c == ' ' || c == '-' || c == '_') && !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 48 {
            break;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() { "note".into() } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let s = NntpMemoryStore::from_url("127.0.0.1:1119").unwrap();
        assert_eq!(s.host(), "127.0.0.1");
        assert_eq!(s.port(), 1119);
        let s2 = NntpMemoryStore::from_url("nntp://localhost:2222").unwrap();
        assert_eq!(s2.host(), "localhost");
        assert_eq!(s2.port(), 2222);
        // Compose DNS hostname:port (must parse; resolve at connect)
        let s3 = NntpMemoryStore::from_url("mkd-gcm-server:1119").unwrap();
        assert_eq!(s3.host(), "mkd-gcm-server");
        assert_eq!(s3.port(), 1119);
    }

    #[test]
    fn msgid_extract() {
        assert_eq!(
            extract_msgid_from_240("240 Article posted <a@b.com>"),
            Some("<a@b.com>".into())
        );
    }

    #[test]
    fn xover_line_parses_classic() {
        let line = "3\tphase1-document-proof.md\tagent <a@b>\t12 Jul 2026 00:00:00 +0000\t<phase1-document-proof-md-v1-v1@lab.example.internal>\t\t120\t4\tXref: local group:3";
        let row = parse_xover_line(line).expect("parse");
        assert_eq!(row.number, 3);
        assert_eq!(row.subject, "phase1-document-proof.md");
        assert_eq!(
            row.message_id,
            "<phase1-document-proof-md-v1-v1@lab.example.internal>"
        );
        assert_eq!(row.bytes, 120);
        assert_eq!(row.lines, 4);
    }
}

#[cfg(test)]
mod oauth_tests {
    use super::encode_oauthbearer_message;
    use base64::Engine;

    #[test]
    fn oauthbearer_roundtrip_shape() {
        let b64 = encode_oauthbearer_message(Some("user@example.com"), "tok");
        let raw = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .unwrap();
        let s = String::from_utf8(raw).unwrap();
        assert!(s.starts_with("n,a=user@example.com,"));
        assert!(s.contains("auth=Bearer tok"));
    }
}

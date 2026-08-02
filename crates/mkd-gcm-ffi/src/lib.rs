// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

//! C ABI over [`mkd_gcm`] for thin language SDKs (Java JNA, Python cffi, etc.).
//!
//! # Ownership
//! - Clients created with [`mkd_gcm_client_create`] must be freed with
//!   [`mkd_gcm_client_destroy`].
//! - Strings returned via out-pointers are heap-allocated C strings; free with
//!   [`mkd_gcm_string_free`].
//! - [`mkd_gcm_last_error`] returns a thread-local static pointer — do not free.
//!
//! # Return codes
//! - `0` success
//! - non-zero failure (see [`mkd_gcm_last_error`])

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use mkd_gcm::{GroupPath, MemoryStore, NntpMemoryStore, PostHeaders};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_error(msg: impl AsRef<str>) {
    let s = CString::new(msg.as_ref()).unwrap_or_else(|_| {
        CString::new("mkd_gcm_ffi: error message contained NUL").expect("static")
    });
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(s));
}

fn clear_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

fn cstr_to_str<'a>(p: *const c_char) -> Result<&'a str, String> {
    if p.is_null() {
        return Err("null string pointer".into());
    }
    // SAFETY: caller must pass valid NUL-terminated C string or null (checked).
    unsafe { CStr::from_ptr(p) }
        .to_str()
        .map_err(|e| format!("invalid UTF-8: {e}"))
}

/// Opaque client handle (heap-allocated).
pub struct MkdGcmClient {
    store: NntpMemoryStore,
}

/// Library version string (static).
#[unsafe(no_mangle)]
pub extern "C" fn mkd_gcm_version() -> *const c_char {
    static V: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    V.as_ptr() as *const c_char
}

/// Last error message for this thread (do not free). Empty string if none.
#[unsafe(no_mangle)]
pub extern "C" fn mkd_gcm_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        if let Some(ref s) = *e.borrow() {
            s.as_ptr()
        } else {
            c"".as_ptr()
        }
    })
}

/// Free a string returned by this library (e.g. message-id out param).
///
/// # Safety
/// `s` must be null or a pointer previously returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: allocated via CString::into_raw in this crate.
    drop(unsafe { CString::from_raw(s) });
}

/// Create a client for `host:port`. Returns null on error.
///
/// # Safety
/// `host` must be a valid UTF-8 C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_create(
    host: *const c_char,
    port: u16,
) -> *mut MkdGcmClient {
    clear_error();
    let host = match cstr_to_str(host) {
        Ok(h) if !h.is_empty() => h,
        Ok(_) => {
            set_error("host is empty");
            return ptr::null_mut();
        }
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
    };
    let store = NntpMemoryStore::new(host, if port == 0 { 1119 } else { port });
    Box::into_raw(Box::new(MkdGcmClient { store }))
}

/// Create a client from URL (`host:port`, `nntp://host:port`). Null on error.
///
/// # Safety
/// `url` must be a valid UTF-8 C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_create_url(url: *const c_char) -> *mut MkdGcmClient {
    clear_error();
    let url = match cstr_to_str(url) {
        Ok(u) => u,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
    };
    match NntpMemoryStore::from_url(url) {
        Ok(store) => Box::into_raw(Box::new(MkdGcmClient { store })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Destroy a client created by `mkd_gcm_client_create*`.
///
/// # Safety
/// `client` must be null or a valid handle from this library (not double-freed).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_destroy(client: *mut MkdGcmClient) {
    if client.is_null() {
        return;
    }
    // SAFETY: exclusive ownership from create.
    drop(unsafe { Box::from_raw(client) });
}

fn client_mut<'a>(client: *mut MkdGcmClient) -> Result<&'a mut MkdGcmClient, String> {
    if client.is_null() {
        return Err("null client".into());
    }
    // SAFETY: caller holds exclusive valid pointer for duration of call.
    Ok(unsafe { &mut *client })
}

/// Set USER/PASS AUTHINFO credentials. Returns 0 on success.
///
/// # Safety
/// Pointers must be valid C strings; client non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_set_user_pass(
    client: *mut MkdGcmClient,
    user: *const c_char,
    pass: *const c_char,
) -> c_int {
    clear_error();
    let c = match client_mut(client) {
        Ok(c) => c,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let user = match cstr_to_str(user) {
        Ok(u) => u,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let pass = match cstr_to_str(pass) {
        Ok(p) => p,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    c.store = c.store.clone().with_auth(user, pass);
    0
}

/// Set OAUTHBEARER / named PAT. `authzid` may be null.
///
/// # Safety
/// `token` valid C string; `authzid` null or valid C string; client non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_set_bearer(
    client: *mut MkdGcmClient,
    token: *const c_char,
    authzid: *const c_char,
) -> c_int {
    clear_error();
    let c = match client_mut(client) {
        Ok(c) => c,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let token = match cstr_to_str(token) {
        Ok(t) if !t.is_empty() => t,
        Ok(_) => {
            set_error("token is empty");
            return -1;
        }
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let mut store = c.store.clone().with_bearer_token(token);
    if !authzid.is_null() {
        match cstr_to_str(authzid) {
            Ok(a) if !a.is_empty() => {
                store = store.with_bearer_authzid(a);
            }
            Ok(_) => {}
            Err(e) => {
                set_error(e);
                return -1;
            }
        }
    }
    c.store = store;
    0
}

/// Default From: header when posts omit it.
///
/// # Safety
/// Valid C strings / client.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_set_from(
    client: *mut MkdGcmClient,
    from: *const c_char,
) -> c_int {
    clear_error();
    let c = match client_mut(client) {
        Ok(c) => c,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let from = match cstr_to_str(from) {
        Ok(f) => f,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    c.store = c.store.clone().with_default_from(from);
    0
}

/// POST a JSON article to `group`. On success, `*out_message_id` is a heap string
/// (free with [`mkd_gcm_string_free`]) or left null if `out_message_id` is null.
///
/// Headers: Content-Type application/json, X-Mkd-Format json, Subject as given.
///
/// # Safety
/// All string pointers valid UTF-8 C strings; client non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_post_json(
    client: *mut MkdGcmClient,
    group: *const c_char,
    subject: *const c_char,
    body_json: *const c_char,
    out_message_id: *mut *mut c_char,
) -> c_int {
    clear_error();
    if !out_message_id.is_null() {
        unsafe { *out_message_id = ptr::null_mut() };
    }
    let c = match client_mut(client) {
        Ok(c) => c,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let group_s = match cstr_to_str(group) {
        Ok(g) => g,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let subject = match cstr_to_str(subject) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let body = match cstr_to_str(body_json) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let gp = match GroupPath::parse(group_s) {
        Ok(g) => g,
        Err(e) => {
            set_error(format!("invalid group: {e}"));
            return -1;
        }
    };
    match c.store.post_json(&gp, subject, body, &PostHeaders::new()) {
        Ok(pref) => {
            if !out_message_id.is_null() {
                match CString::new(pref.message_id) {
                    Ok(cs) => unsafe { *out_message_id = cs.into_raw() },
                    Err(_) => {
                        set_error("message-id contained NUL");
                        return -1;
                    }
                }
            }
            0
        }
        Err(e) => {
            set_error(e.to_string());
            -1
        }
    }
}

/// Generic POST: optional content_type (null → text/plain path via core defaults).
///
/// # Safety
/// C string pointers; client non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkd_gcm_client_post(
    client: *mut MkdGcmClient,
    group: *const c_char,
    subject: *const c_char,
    content_type: *const c_char,
    body: *const c_char,
    out_message_id: *mut *mut c_char,
) -> c_int {
    clear_error();
    if !out_message_id.is_null() {
        unsafe { *out_message_id = ptr::null_mut() };
    }
    let c = match client_mut(client) {
        Ok(c) => c,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let group_s = match cstr_to_str(group) {
        Ok(g) => g,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let subject = match cstr_to_str(subject) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let body = match cstr_to_str(body) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return -1;
        }
    };
    let gp = match GroupPath::parse(group_s) {
        Ok(g) => g,
        Err(e) => {
            set_error(format!("invalid group: {e}"));
            return -1;
        }
    };
    let mut headers = PostHeaders::new().with("Subject", subject);
    if !content_type.is_null() {
        match cstr_to_str(content_type) {
            Ok(ct) if !ct.is_empty() => {
                headers.set("Content-Type", ct);
                if ct.to_ascii_lowercase().contains("json") {
                    headers.set("X-Mkd-Format", "json");
                }
            }
            Ok(_) => {}
            Err(e) => {
                set_error(e);
                return -1;
            }
        }
    }
    match c.store.post(&gp, &headers, body) {
        Ok(pref) => {
            if !out_message_id.is_null() {
                match CString::new(pref.message_id) {
                    Ok(cs) => unsafe { *out_message_id = cs.into_raw() },
                    Err(_) => {
                        set_error("message-id contained NUL");
                        return -1;
                    }
                }
            }
            0
        }
        Err(e) => {
            set_error(e.to_string());
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn create_destroy_and_error_path() {
        let host = CString::new("127.0.0.1").unwrap();
        let c = unsafe { mkd_gcm_client_create(host.as_ptr(), 1119) };
        assert!(!c.is_null());
        let token = CString::new("not-a-real-token").unwrap();
        assert_eq!(
            unsafe { mkd_gcm_client_set_bearer(c, token.as_ptr(), ptr::null()) },
            0
        );
        // Post will fail (no server) but should not crash
        let group =
            CString::new("mkd.gcm.orgs.org.example.language.projects.crowdsource-demo.inbound")
                .unwrap();
        let subj = CString::new("test").unwrap();
        let body = CString::new(r#"{"email":"a@b.co"}"#).unwrap();
        let mut mid: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            mkd_gcm_client_post_json(c, group.as_ptr(), subj.as_ptr(), body.as_ptr(), &mut mid)
        };
        assert_ne!(rc, 0);
        let err = unsafe { CStr::from_ptr(mkd_gcm_last_error()) }
            .to_string_lossy()
            .to_string();
        assert!(!err.is_empty(), "expected last_error");
        unsafe { mkd_gcm_client_destroy(c) };
    }

    #[test]
    fn version_nonempty() {
        let v = unsafe { CStr::from_ptr(mkd_gcm_version()) }
            .to_string_lossy()
            .to_string();
        assert!(!v.is_empty());
    }
}

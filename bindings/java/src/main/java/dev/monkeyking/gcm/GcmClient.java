// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.PointerByReference;

/**
 * Thin JVM client over Rust {@code mkd-gcm} (GCM network).
 *
 * <p>Typical host application flow:
 *
 * <pre>
 *   GcmClient c = GcmClient.connect(host, port);
 *   c.setBearerToken(tokenFromServerSecrets);
 *   c.postCorrection(inboundGroup, submissionFromBrowserJson);
 * </pre>
 *
 * <p>Native library name: {@code mkd_gcm_ffi}. Prefer depending on
 * {@code mkd-gcm-natives} (extracted automatically), or set {@code
 * jna.library.path} to a directory that contains the shared library.
 */
public final class GcmClient implements AutoCloseable {

    private Pointer handle;
    private boolean closed;

    private GcmClient(Pointer handle) {
        if (handle == null || Pointer.NULL.equals(handle)) {
            throw GcmException.fromNative("create");
        }
        this.handle = handle;
    }

    /** Connect to host:port (default GCM port 1119 if port is 0). */
    public static GcmClient connect(String host, int port) {
        try {
            Pointer p = NativeLib.INSTANCE.mkd_gcm_client_create(host, (short) port);
            if (p == null) {
                throw GcmException.fromNative("create");
            }
            return new GcmClient(p);
        } catch (UnsatisfiedLinkError e) {
            throw new GcmException(
                    "native library mkd_gcm_ffi not found — add dependency "
                            + "dev.monkeyking:mkd-gcm-natives or set jna.library.path",
                    e);
        }
    }

    /** Connect using {@code host:port} or a GCM URL form. */
    public static GcmClient connectUrl(String url) {
        try {
            Pointer p = NativeLib.INSTANCE.mkd_gcm_client_create_url(url);
            if (p == null) {
                throw GcmException.fromNative("create_url");
            }
            return new GcmClient(p);
        } catch (UnsatisfiedLinkError e) {
            throw new GcmException(
                    "native library mkd_gcm_ffi not found — add dependency "
                            + "dev.monkeyking:mkd-gcm-natives or set jna.library.path",
                    e);
        }
    }

    public static String nativeVersion() {
        try {
            return NativeLib.INSTANCE.mkd_gcm_version();
        } catch (UnsatisfiedLinkError e) {
            return null;
        }
    }

    /** Password auth (lab / password principals). */
    public void setUserPass(String user, String pass) {
        ensureOpen();
        if (NativeLib.INSTANCE.mkd_gcm_client_set_user_pass(handle, user, pass) != 0) {
            throw GcmException.fromNative("set_user_pass");
        }
    }

    /**
     * Named PAT or session token via SASL OAUTHBEARER.
     * Preferred for project-submit tokens issued by your GCM operator surface.
     */
    public void setBearerToken(String token) {
        setBearerToken(token, null);
    }

    public void setBearerToken(String token, String authzid) {
        ensureOpen();
        if (NativeLib.INSTANCE.mkd_gcm_client_set_bearer(handle, token, authzid) != 0) {
            throw GcmException.fromNative("set_bearer");
        }
    }

    public void setDefaultFrom(String from) {
        ensureOpen();
        if (NativeLib.INSTANCE.mkd_gcm_client_set_from(handle, from) != 0) {
            throw GcmException.fromNative("set_from");
        }
    }

    /**
     * POST JSON body to group (Content-Type application/json).
     *
     * @return Message-ID from the server when available
     */
    public String postJson(String group, String subject, String bodyJson) {
        ensureOpen();
        PointerByReference out = new PointerByReference();
        int rc =
                NativeLib.INSTANCE.mkd_gcm_client_post_json(
                        handle, group, subject, bodyJson, out);
        if (rc != 0) {
            throw GcmException.fromNative("post_json");
        }
        return takeString(out);
    }

    /**
     * POST a language correction (or any structured JSON) using the shared
     * {@link CorrectionSubmission} browser-plugin JSON contract.
     */
    public String postCorrection(String group, CorrectionSubmission submission) {
        if (submission == null) {
            throw new GcmException("submission is null");
        }
        submission.requireEmail();
        String subject =
                submission.messageId != null && !submission.messageId.isEmpty()
                        ? "correction: " + submission.messageId
                        : "language correction";
        if (submission.email != null && !submission.email.isEmpty()) {
            setDefaultFrom(submission.email);
        }
        return postJson(group, subject, submission.toJson());
    }

    /** Generic post with optional content type (null for default). */
    public String post(String group, String subject, String contentType, String body) {
        ensureOpen();
        PointerByReference out = new PointerByReference();
        int rc =
                NativeLib.INSTANCE.mkd_gcm_client_post(
                        handle, group, subject, contentType, body, out);
        if (rc != 0) {
            throw GcmException.fromNative("post");
        }
        return takeString(out);
    }

    private static String takeString(PointerByReference out) {
        Pointer p = out.getValue();
        if (p == null) {
            return null;
        }
        try {
            return p.getString(0);
        } finally {
            NativeLib.INSTANCE.mkd_gcm_string_free(p);
        }
    }

    private void ensureOpen() {
        if (closed || handle == null) {
            throw new GcmException("client is closed");
        }
    }

    @Override
    public void close() {
        if (!closed && handle != null) {
            NativeLib.INSTANCE.mkd_gcm_client_destroy(handle);
            handle = null;
            closed = true;
        }
    }
}

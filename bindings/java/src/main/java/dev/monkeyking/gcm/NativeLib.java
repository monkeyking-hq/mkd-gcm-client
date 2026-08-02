// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.PointerByReference;

/**
 * JNA mapping to {@code libmkd_gcm_ffi} ({@code mkd_gcm.h}).
 *
 * <p>Load path: {@code jna.library.path} or system library path must contain
 * {@code mkd_gcm_ffi} (e.g. {@code target/release} after {@code cargo build -p mkd-gcm-ffi}).
 */
interface NativeLib extends Library {

    NativeLib INSTANCE = Native.load("mkd_gcm_ffi", NativeLib.class);

    String mkd_gcm_version();

    String mkd_gcm_last_error();

    void mkd_gcm_string_free(Pointer s);

    Pointer mkd_gcm_client_create(String host, short port);

    Pointer mkd_gcm_client_create_url(String url);

    void mkd_gcm_client_destroy(Pointer client);

    int mkd_gcm_client_set_user_pass(Pointer client, String user, String pass);

    int mkd_gcm_client_set_bearer(Pointer client, String token, String authzid);

    int mkd_gcm_client_set_from(Pointer client, String from);

    int mkd_gcm_client_post_json(
            Pointer client,
            String group,
            String subject,
            String bodyJson,
            PointerByReference outMessageId);

    int mkd_gcm_client_post(
            Pointer client,
            String group,
            String subject,
            String contentType,
            String body,
            PointerByReference outMessageId);
}

// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

/** Failure from the native mkd-gcm client (configuration or GCM transport). */
public class GcmException extends RuntimeException {

    public GcmException(String message) {
        super(message);
    }

    public GcmException(String message, Throwable cause) {
        super(message, cause);
    }

    static GcmException fromNative(String op) {
        String err = "";
        try {
            err = NativeLib.INSTANCE.mkd_gcm_last_error();
        } catch (Throwable ignored) {
            // library missing
        }
        if (err == null || err.isEmpty()) {
            return new GcmException(op + " failed");
        }
        return new GcmException(op + ": " + err);
    }
}

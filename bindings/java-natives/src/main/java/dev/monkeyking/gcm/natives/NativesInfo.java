// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm.natives;

/**
 * Marker type for the {@code mkd-gcm-natives} artifact.
 *
 * <p>Shared libraries are packaged as classpath resources under {@code
 * dev/monkeyking/gcm/native/&lt;platform&gt;/} and loaded by {@code mkd-gcm-sdk}.
 */
public final class NativesInfo {

    /** Maven artifact id. */
    public static final String ARTIFACT_ID = "mkd-gcm-natives";

    private NativesInfo() {}
}

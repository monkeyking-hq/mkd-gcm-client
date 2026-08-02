// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.Locale;

/**
 * Locates {@code mkd_gcm_ffi} for JNA: system path first, then classpath
 * resources shipped by {@code mkd-gcm-natives}.
 *
 * <p>Resource layout: {@code dev/monkeyking/gcm/native/<platform>/<libfile>}
 * where platform matches os-maven-plugin classifiers (e.g. {@code windows-x86_64}).
 */
final class NativeLibLoader {

    private static final String RESOURCE_PREFIX = "dev/monkeyking/gcm/native/";
    private static volatile boolean prepared;

    private NativeLibLoader() {}

    /** Idempotent; safe to call before {@code Native.load("mkd_gcm_ffi", ...)}. */
    static synchronized void prepare() {
        if (prepared) {
            return;
        }
        // Honour an explicit jna.library.path set by the host application / installer.
        // (Do not treat java.library.path as a signal — the JVM always sets it.)
        if (nonEmpty(System.getProperty("jna.library.path"))) {
            prepared = true;
            return;
        }

        Platform plat = Platform.detect();
        String resource = RESOURCE_PREFIX + plat.classifier + "/" + plat.libFile;
        ClassLoader cl = NativeLibLoader.class.getClassLoader();
        InputStream in = cl != null ? cl.getResourceAsStream(resource) : null;
        if (in == null) {
            in = ClassLoader.getSystemResourceAsStream(resource);
        }
        if (in == null) {
            // Leave load to JNA system search; connect() will surface a clear error.
            prepared = true;
            return;
        }

        try {
            Path dir = Files.createTempDirectory("mkd-gcm-native-");
            dir.toFile().deleteOnExit();
            Path lib = dir.resolve(plat.libFile);
            try (InputStream src = in;
                    OutputStream out =
                            Files.newOutputStream(
                                    lib, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING)) {
                src.transferTo(out);
            }
            lib.toFile().deleteOnExit();

            String existing = System.getProperty("jna.library.path");
            if (nonEmpty(existing)) {
                System.setProperty("jna.library.path", dir + java.io.File.pathSeparator + existing);
            } else {
                System.setProperty("jna.library.path", dir.toString());
            }
        } catch (IOException e) {
            throw new UnsatisfiedLinkError(
                    "failed to extract mkd_gcm_ffi from classpath resource " + resource + ": " + e);
        } finally {
            prepared = true;
        }
    }

    private static boolean nonEmpty(String s) {
        return s != null && !s.isBlank();
    }

    static final class Platform {
        final String classifier;
        final String libFile;

        Platform(String classifier, String libFile) {
            this.classifier = classifier;
            this.libFile = libFile;
        }

        static Platform detect() {
            String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
            String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
            String normArch;
            if (arch.equals("amd64") || arch.equals("x86_64")) {
                normArch = "x86_64";
            } else if (arch.equals("aarch64") || arch.equals("arm64")) {
                // os-maven-plugin uses aarch_64
                normArch = "aarch_64";
            } else if (arch.equals("x86") || arch.equals("i386") || arch.equals("i686")) {
                normArch = "x86";
            } else {
                normArch = arch.replace('-', '_');
            }

            if (os.contains("win")) {
                return new Platform("windows-" + normArch, "mkd_gcm_ffi.dll");
            }
            if (os.contains("mac") || os.contains("darwin")) {
                return new Platform("osx-" + normArch, "libmkd_gcm_ffi.dylib");
            }
            // Linux and other Unix
            return new Platform("linux-" + normArch, "libmkd_gcm_ffi.so");
        }
    }
}

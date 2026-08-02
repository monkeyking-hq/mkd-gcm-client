// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm.jackson2;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import dev.monkeyking.gcm.CorrectionSubmission;

/**
 * Optional Jackson 2.x codec for {@link CorrectionSubmission}.
 *
 * <p>Depends on {@code mkd-gcm-sdk} + {@code jackson-databind} 2.x.
 */
public final class CorrectionJson {

    private static final ObjectMapper MAPPER =
            new ObjectMapper()
                    .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
                    .configure(SerializationFeature.FAIL_ON_EMPTY_BEANS, false);

    private CorrectionJson() {}

    public static ObjectMapper mapper() {
        return MAPPER;
    }

    public static String toJson(CorrectionSubmission s) {
        try {
            return MAPPER.writeValueAsString(s);
        } catch (Exception e) {
            throw new IllegalArgumentException("serialize CorrectionSubmission", e);
        }
    }

    public static CorrectionSubmission fromJson(String json) {
        try {
            return MAPPER.readValue(json, CorrectionSubmission.class);
        } catch (Exception e) {
            throw new IllegalArgumentException("deserialize CorrectionSubmission", e);
        }
    }
}

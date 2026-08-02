// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm.jackson3;

import dev.monkeyking.gcm.CorrectionSubmission;
import tools.jackson.databind.DeserializationFeature;
import tools.jackson.databind.ObjectMapper;
import tools.jackson.databind.json.JsonMapper;

/**
 * Optional Jackson 3.x codec for {@link CorrectionSubmission}.
 *
 * <p>Depends on {@code mkd-gcm-sdk} + Jackson 3 databind. If Jackson 3 package
 * coordinates change at GA, update this module's pom accordingly.
 */
public final class CorrectionJson {

    private static final ObjectMapper MAPPER =
            JsonMapper.builder()
                    .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
                    .build();

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

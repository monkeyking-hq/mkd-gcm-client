// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm.jackson3;

import static org.junit.jupiter.api.Assertions.assertEquals;

import dev.monkeyking.gcm.CorrectionSubmission;
import org.junit.jupiter.api.Test;

class CorrectionJsonTest {

    @Test
    void roundTrip() {
        CorrectionSubmission s = new CorrectionSubmission();
        s.email = "user@example.com";
        s.locale = "en";
        s.proposedText = "Hello";
        String json = CorrectionJson.toJson(s);
        CorrectionSubmission back = CorrectionJson.fromJson(json);
        assertEquals("user@example.com", back.email);
        assertEquals("Hello", back.proposedText);
    }
}

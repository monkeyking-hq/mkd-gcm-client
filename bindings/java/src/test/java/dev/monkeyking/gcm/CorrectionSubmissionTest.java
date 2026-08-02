// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class CorrectionSubmissionTest {

    @Test
    void toJsonIncludesEmailAndCamelCaseKeys() {
        CorrectionSubmission s = new CorrectionSubmission();
        s.email = "user@example.com";
        s.locale = "en-US";
        s.proposedText = "Save \"now\"";
        s.messageId = "app.save";
        s.notes = "";
        s.submittedAt = "2026-08-01T00:00:00.000Z";
        s.source = new CorrectionSubmission.Source();
        s.source.tagName = "button";
        s.source.matchReason = "button";
        s.source.pageUrl = "https://example.com/";

        String json = s.toJson();
        assertTrue(json.contains("\"email\":\"user@example.com\""));
        assertTrue(json.contains("\"proposedText\""));
        assertTrue(json.contains("\\\"now\\\"") || json.contains("Save"));
        assertTrue(json.contains("\"messageId\":\"app.save\""));
    }

    @Test
    void requireEmail() {
        CorrectionSubmission s = new CorrectionSubmission();
        assertThrows(IllegalArgumentException.class, s::requireEmail);
    }
}

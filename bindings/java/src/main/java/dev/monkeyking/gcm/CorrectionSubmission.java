// Copyright (c) 2026 MonkeyKing.dev
//
// SPDX-License-Identifier: MIT

package dev.monkeyking.gcm;

import java.util.Objects;

/**
 * Language-correction submission payload (JSON field names are camelCase to
 * match the browser language plugin contract).
 *
 * <p>Host REST endpoints should deserialize the plugin body into this type (or
 * an equivalent model) before calling {@link GcmClient#postCorrection}.
 *
 * <p>For Jackson 2.x / 3.x, see optional modules {@code mkd-gcm-sdk-jackson2}
 * and {@code mkd-gcm-sdk-jackson3}. Core has no Jackson dependency.
 */
public final class CorrectionSubmission {

    public String currentText;
    public String proposedText;
    public String currentAriaLabel;
    public String proposedAriaLabel;
    public String ariaLabelledby;
    public String currentTitle;
    public String messageId;
    public String notes;
    /** Required for GCM inbound (reply-to / attribution). */
    public String email;
    public String locale;
    public Source source;
    public String submittedAt;

    public static final class Source {
        public String tagName;
        public String matchReason;
        public String elementId;
        public String pageUrl;
    }

    /**
     * Minimal JSON serializer (no Jackson required). Hosts that already use
     * Jackson may prefer the optional jackson2/jackson3 helpers.
     */
    public String toJson() {
        StringBuilder sb = new StringBuilder(256);
        sb.append('{');
        field(sb, "currentText", currentText, true);
        field(sb, "proposedText", proposedText, false);
        field(sb, "currentAriaLabel", currentAriaLabel, false);
        field(sb, "proposedAriaLabel", proposedAriaLabel, false);
        field(sb, "ariaLabelledby", ariaLabelledby, false);
        field(sb, "currentTitle", currentTitle, false);
        field(sb, "messageId", messageId, false);
        field(sb, "notes", notes == null ? "" : notes, false);
        field(sb, "email", email, false);
        field(sb, "locale", locale, false);
        sb.append(",\"source\":");
        if (source == null) {
            sb.append("null");
        } else {
            sb.append('{');
            field(sb, "tagName", source.tagName, true);
            field(sb, "matchReason", source.matchReason, false);
            field(sb, "elementId", source.elementId, false);
            field(sb, "pageUrl", source.pageUrl, false);
            sb.append('}');
        }
        field(sb, "submittedAt", submittedAt, false);
        sb.append('}');
        return sb.toString();
    }

    private static void field(StringBuilder sb, String name, String value, boolean first) {
        if (!first) {
            sb.append(',');
        }
        sb.append('"').append(name).append("\":");
        if (value == null) {
            sb.append("null");
        } else {
            sb.append('"').append(escape(value)).append('"');
        }
    }

    private static String escape(String s) {
        StringBuilder o = new StringBuilder(s.length() + 8);
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '\\':
                    o.append("\\\\");
                    break;
                case '"':
                    o.append("\\\"");
                    break;
                case '\n':
                    o.append("\\n");
                    break;
                case '\r':
                    o.append("\\r");
                    break;
                case '\t':
                    o.append("\\t");
                    break;
                default:
                    if (c < 0x20) {
                        o.append(String.format("\\u%04x", (int) c));
                    } else {
                        o.append(c);
                    }
            }
        }
        return o.toString();
    }

    public void requireEmail() {
        if (email == null || email.trim().isEmpty()) {
            throw new IllegalArgumentException("email is required");
        }
    }

    @Override
    public String toString() {
        return "CorrectionSubmission{email=" + email + ", messageId=" + messageId + "}";
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof CorrectionSubmission)) {
            return false;
        }
        CorrectionSubmission that = (CorrectionSubmission) o;
        return Objects.equals(email, that.email)
                && Objects.equals(messageId, that.messageId)
                && Objects.equals(proposedText, that.proposedText);
    }

    @Override
    public int hashCode() {
        return Objects.hash(email, messageId, proposedText);
    }
}

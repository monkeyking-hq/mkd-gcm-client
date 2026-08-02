/* Copyright (c) 2026 MonkeyKing.dev */
/* SPDX-License-Identifier: MIT */

/**
 * C ABI for mkd-gcm (crate mkd-gcm-ffi).
 *
 * Thin language SDKs (Java/JNA, Python, .NET) link against libmkd_gcm_ffi.
 *
 * Ownership:
 *   - mkd_gcm_client_create* → mkd_gcm_client_destroy
 *   - out_message_id strings → mkd_gcm_string_free
 *   - mkd_gcm_last_error / mkd_gcm_version → do not free
 *
 * Returns: 0 success, non-zero failure (see mkd_gcm_last_error).
 */

#ifndef MKD_GCM_H
#define MKD_GCM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MkdGcmClient MkdGcmClient;

const char *mkd_gcm_version(void);
const char *mkd_gcm_last_error(void);
void mkd_gcm_string_free(char *s);

MkdGcmClient *mkd_gcm_client_create(const char *host, uint16_t port);
MkdGcmClient *mkd_gcm_client_create_url(const char *url);
void mkd_gcm_client_destroy(MkdGcmClient *client);

int mkd_gcm_client_set_user_pass(MkdGcmClient *client, const char *user, const char *pass);
int mkd_gcm_client_set_bearer(MkdGcmClient *client, const char *token, const char *authzid);
int mkd_gcm_client_set_from(MkdGcmClient *client, const char *from);

int mkd_gcm_client_post_json(
    MkdGcmClient *client,
    const char *group,
    const char *subject,
    const char *body_json,
    char **out_message_id);

int mkd_gcm_client_post(
    MkdGcmClient *client,
    const char *group,
    const char *subject,
    const char *content_type,
    const char *body,
    char **out_message_id);

#ifdef __cplusplus
}
#endif

#endif /* MKD_GCM_H */

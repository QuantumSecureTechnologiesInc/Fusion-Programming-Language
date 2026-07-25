#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#ifndef _CRT_SECURE_NO_WARNINGS
#define _CRT_SECURE_NO_WARNINGS
#endif
#else
#ifndef _strdup
#define _strdup strdup
#endif
#endif
/* ================================================================
 * Handle-based HashMap runtime
 * Structs are small (handle + size ≤ 16 bytes); real data lives here.
 * ================================================================ */
#define HM_CAP  256
#define HM_POOL 64

typedef struct {
    int   keys[HM_CAP];
    int   values[HM_CAP];
    int   states[HM_CAP]; /* 0=empty 1=occupied 2=deleted */
    int   size;
    int   alive;
} HMapII;

typedef struct {
    int    keys[HM_CAP];
    char*  values[HM_CAP];
    int    states[HM_CAP];
    int    size;
    int    alive;
} HMapIS;

typedef struct {
    char*  keys[HM_CAP];
    char*  values[HM_CAP];
    int    states[HM_CAP];
    int    size;
    int    alive;
} HMapSS;

static HMapII hmii_pool[HM_POOL];
static HMapIS hmis_pool[HM_POOL];
static HMapSS hmss_pool[HM_POOL];

static unsigned int hm_hash_int(int k) {
    unsigned int h = (unsigned int)k * 2654435761u;
    return h % HM_CAP;
}

/* --- HashMapIntInt --- */
int fusion_hmii_create(void) {
    for (int i = 1; i < HM_POOL; i++) {
        if (!hmii_pool[i].alive) {
            memset(&hmii_pool[i], 0, sizeof(HMapII));
            hmii_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_hmii_destroy(int h) {
    if (h > 0 && h < HM_POOL) hmii_pool[h].alive = 0;
}
int fusion_hmii_size(int h) {
    return (h > 0 && h < HM_POOL && hmii_pool[h].alive) ? hmii_pool[h].size : 0;
}
void fusion_hmii_insert(int h, int key, int value) {
    if (h <= 0 || h >= HM_POOL || !hmii_pool[h].alive) return;
    HMapII* m = &hmii_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0 || m->states[pos] == 2) {
            m->keys[pos] = key;
            m->values[pos] = value;
            m->states[pos] = 1;
            m->size++;
            return;
        }
        if (m->states[pos] == 1 && m->keys[pos] == key) {
            m->values[pos] = value;
            return;
        }
    }
}
int fusion_hmii_get(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmii_pool[h].alive) return 0;
    HMapII* m = &hmii_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return 0;
        if (m->states[pos] == 1 && m->keys[pos] == key) return m->values[pos];
    }
    return 0;
}
int fusion_hmii_contains(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmii_pool[h].alive) return 0;
    HMapII* m = &hmii_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return 0;
        if (m->states[pos] == 1 && m->keys[pos] == key) return 1;
    }
    return 0;
}
void fusion_hmii_remove(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmii_pool[h].alive) return;
    HMapII* m = &hmii_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return;
        if (m->states[pos] == 1 && m->keys[pos] == key) {
            m->states[pos] = 2;
            m->size--;
            return;
        }
    }
}

/* --- HashMapIntString --- */
int fusion_hmis_create(void) {
    for (int i = 1; i < HM_POOL; i++) {
        if (!hmis_pool[i].alive) {
            memset(&hmis_pool[i], 0, sizeof(HMapIS));
            hmis_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_hmis_destroy(int h) {
    if (h > 0 && h < HM_POOL) {
        for (int i = 0; i < HM_CAP; i++) {
            if (hmis_pool[h].values[i]) { free(hmis_pool[h].values[i]); hmis_pool[h].values[i] = NULL; }
        }
        hmis_pool[h].alive = 0;
    }
}
int fusion_hmis_size(int h) {
    return (h > 0 && h < HM_POOL && hmis_pool[h].alive) ? hmis_pool[h].size : 0;
}
void fusion_hmis_insert(int h, int key, const char* value) {
    if (h <= 0 || h >= HM_POOL || !hmis_pool[h].alive) return;
    HMapIS* m = &hmis_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0 || m->states[pos] == 2) {
            m->keys[pos] = key;
            if (m->values[pos]) free(m->values[pos]);
            m->values[pos] = value ? _strdup(value) : NULL;
            m->states[pos] = 1;
            m->size++;
            return;
        }
        if (m->states[pos] == 1 && m->keys[pos] == key) {
            if (m->values[pos]) free(m->values[pos]);
            m->values[pos] = value ? _strdup(value) : NULL;
            return;
        }
    }
}
const char* fusion_hmis_get(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmis_pool[h].alive) return "";
    HMapIS* m = &hmis_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return "";
        if (m->states[pos] == 1 && m->keys[pos] == key) return m->values[pos] ? m->values[pos] : "";
    }
    return "";
}
int fusion_hmis_contains(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmis_pool[h].alive) return 0;
    HMapIS* m = &hmis_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return 0;
        if (m->states[pos] == 1 && m->keys[pos] == key) return 1;
    }
    return 0;
}
void fusion_hmis_remove(int h, int key) {
    if (h <= 0 || h >= HM_POOL || !hmis_pool[h].alive) return;
    HMapIS* m = &hmis_pool[h];
    unsigned int idx = hm_hash_int(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return;
        if (m->states[pos] == 1 && m->keys[pos] == key) {
            m->states[pos] = 2;
            m->size--;
            return;
        }
    }
}

/* --- HashMapStringString --- */
static unsigned int hm_hash_str(const char* s) {
    unsigned int h = 5381;
    if (s) { while (*s) { h = ((h << 5) + h) + (unsigned char)*s; s++; } }
    return h % HM_CAP;
}

int fusion_hmss_create(void) {
    for (int i = 1; i < HM_POOL; i++) {
        if (!hmss_pool[i].alive) {
            memset(&hmss_pool[i], 0, sizeof(HMapSS));
            hmss_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_hmss_destroy(int h) {
    if (h > 0 && h < HM_POOL) {
        for (int i = 0; i < HM_CAP; i++) {
            if (hmss_pool[h].keys[i])   { free(hmss_pool[h].keys[i]);   hmss_pool[h].keys[i] = NULL; }
            if (hmss_pool[h].values[i]) { free(hmss_pool[h].values[i]); hmss_pool[h].values[i] = NULL; }
        }
        hmss_pool[h].alive = 0;
    }
}
int fusion_hmss_size(int h) {
    return (h > 0 && h < HM_POOL && hmss_pool[h].alive) ? hmss_pool[h].size : 0;
}
void fusion_hmss_insert(int h, const char* key, const char* value) {
    if (h <= 0 || h >= HM_POOL || !hmss_pool[h].alive) return;
    HMapSS* m = &hmss_pool[h];
    unsigned int idx = hm_hash_str(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0 || m->states[pos] == 2) {
            m->keys[pos] = key ? _strdup(key) : NULL;
            m->values[pos] = value ? _strdup(value) : NULL;
            m->states[pos] = 1;
            m->size++;
            return;
        }
        if (m->states[pos] == 1 && m->keys[pos] && strcmp(m->keys[pos], key) == 0) {
            if (m->values[pos]) free(m->values[pos]);
            m->values[pos] = value ? _strdup(value) : NULL;
            return;
        }
    }
}
const char* fusion_hmss_get(int h, const char* key) {
    if (h <= 0 || h >= HM_POOL || !hmss_pool[h].alive) return "";
    HMapSS* m = &hmss_pool[h];
    unsigned int idx = hm_hash_str(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return "";
        if (m->states[pos] == 1 && m->keys[pos] && strcmp(m->keys[pos], key) == 0)
            return m->values[pos] ? m->values[pos] : "";
    }
    return "";
}
int fusion_hmss_contains(int h, const char* key) {
    if (h <= 0 || h >= HM_POOL || !hmss_pool[h].alive) return 0;
    HMapSS* m = &hmss_pool[h];
    unsigned int idx = hm_hash_str(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return 0;
        if (m->states[pos] == 1 && m->keys[pos] && strcmp(m->keys[pos], key) == 0) return 1;
    }
    return 0;
}
void fusion_hmss_remove(int h, const char* key) {
    if (h <= 0 || h >= HM_POOL || !hmss_pool[h].alive) return;
    HMapSS* m = &hmss_pool[h];
    unsigned int idx = hm_hash_str(key);
    for (int i = 0; i < HM_CAP; i++) {
        int pos = (idx + i) % HM_CAP;
        if (m->states[pos] == 0) return;
        if (m->states[pos] == 1 && m->keys[pos] && strcmp(m->keys[pos], key) == 0) {
            free(m->keys[pos]); m->keys[pos] = NULL;
            free(m->values[pos]); m->values[pos] = NULL;
            m->states[pos] = 2;
            m->size--;
            return;
        }
    }
}
/* runtime/hashset_runtime.c — Handle-based HashSet runtime for Fusion.
 * Two set types: HashSetInt, HashSetString.
 * Pool capacity: 64 sets, 256 slots each. Linear probing.
 */
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

#define HS_CAP  256
#define HS_POOL 64

/* --- HashSetInt --- */
typedef struct {
    int   data[HS_CAP];
    int   states[HS_CAP]; /* 0=empty 1=occupied 2=deleted */
    int   size;
    int   alive;
} HSetInt;

static HSetInt hsi_pool[HS_POOL];

static unsigned int hs_hash_int(int k) {
    unsigned int h = (unsigned int)k * 2654435761u;
    return h % HS_CAP;
}

int fusion_hsi_create(void) {
    for (int i = 1; i < HS_POOL; i++) {
        if (!hsi_pool[i].alive) {
            memset(&hsi_pool[i], 0, sizeof(HSetInt));
            hsi_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_hsi_destroy(int h) {
    if (h > 0 && h < HS_POOL) hsi_pool[h].alive = 0;
}
int fusion_hsi_size(int h) {
    return (h > 0 && h < HS_POOL && hsi_pool[h].alive) ? hsi_pool[h].size : 0;
}
int fusion_hsi_is_empty(int h) {
    return fusion_hsi_size(h) == 0;
}
void fusion_hsi_insert(int h, int value) {
    if (h <= 0 || h >= HS_POOL || !hsi_pool[h].alive) return;
    HSetInt* s = &hsi_pool[h];
    unsigned int idx = hs_hash_int(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) {
            s->data[pos] = value;
            s->states[pos] = 1;
            s->size++;
            return;
        }
        if (s->states[pos] == 1 && s->data[pos] == value) return; /* already present */
        if (s->states[pos] == 2) {
            s->data[pos] = value;
            s->states[pos] = 1;
            s->size++;
            return;
        }
    }
}
int fusion_hsi_contains(int h, int value) {
    if (h <= 0 || h >= HS_POOL || !hsi_pool[h].alive) return 0;
    HSetInt* s = &hsi_pool[h];
    unsigned int idx = hs_hash_int(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) return 0;
        if (s->states[pos] == 1 && s->data[pos] == value) return 1;
    }
    return 0;
}
void fusion_hsi_remove(int h, int value) {
    if (h <= 0 || h >= HS_POOL || !hsi_pool[h].alive) return;
    HSetInt* s = &hsi_pool[h];
    unsigned int idx = hs_hash_int(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) return;
        if (s->states[pos] == 1 && s->data[pos] == value) {
            s->states[pos] = 2;
            s->size--;
            return;
        }
    }
}
void fusion_hsi_clear(int h) {
    if (h <= 0 || h >= HS_POOL || !hsi_pool[h].alive) return;
    HSetInt* s = &hsi_pool[h];
    memset(s->states, 0, sizeof(s->states));
    s->size = 0;
}

/* --- HashSetString --- */
typedef struct {
    char* data[HS_CAP];
    int   states[HS_CAP];
    int   size;
    int   alive;
} HSetStr;

static HSetStr hss_pool[HS_POOL];

static unsigned int hs_hash_str(const char* s) {
    unsigned int h = 5381;
    if (s) { while (*s) { h = ((h << 5) + h) + (unsigned char)*s; s++; } }
    return h % HS_CAP;
}

int fusion_hss_create(void) {
    for (int i = 1; i < HS_POOL; i++) {
        if (!hss_pool[i].alive) {
            memset(&hss_pool[i], 0, sizeof(HSetStr));
            hss_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_hss_destroy(int h) {
    if (h > 0 && h < HS_POOL) {
        for (int i = 0; i < HS_CAP; i++) {
            if (hss_pool[h].data[i]) { free(hss_pool[h].data[i]); hss_pool[h].data[i] = NULL; }
        }
        hss_pool[h].alive = 0;
    }
}
int fusion_hss_size(int h) {
    return (h > 0 && h < HS_POOL && hss_pool[h].alive) ? hss_pool[h].size : 0;
}
int fusion_hss_is_empty(int h) {
    return fusion_hss_size(h) == 0;
}
void fusion_hss_insert(int h, const char* value) {
    if (h <= 0 || h >= HS_POOL || !hss_pool[h].alive) return;
    HSetStr* s = &hss_pool[h];
    unsigned int idx = hs_hash_str(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) {
            s->data[pos] = value ? _strdup(value) : NULL;
            s->states[pos] = 1;
            s->size++;
            return;
        }
        if (s->states[pos] == 1 && s->data[pos] && value && strcmp(s->data[pos], value) == 0) return;
        if (s->states[pos] == 2) {
            s->data[pos] = value ? _strdup(value) : NULL;
            s->states[pos] = 1;
            s->size++;
            return;
        }
    }
}
int fusion_hss_contains(int h, const char* value) {
    if (h <= 0 || h >= HS_POOL || !hss_pool[h].alive) return 0;
    HSetStr* s = &hss_pool[h];
    unsigned int idx = hs_hash_str(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) return 0;
        if (s->states[pos] == 1 && s->data[pos] && value && strcmp(s->data[pos], value) == 0) return 1;
    }
    return 0;
}
void fusion_hss_remove(int h, const char* value) {
    if (h <= 0 || h >= HS_POOL || !hss_pool[h].alive) return;
    HSetStr* s = &hss_pool[h];
    unsigned int idx = hs_hash_str(value);
    for (int i = 0; i < HS_CAP; i++) {
        int pos = (idx + i) % HS_CAP;
        if (s->states[pos] == 0) return;
        if (s->states[pos] == 1 && s->data[pos] && value && strcmp(s->data[pos], value) == 0) {
            free(s->data[pos]); s->data[pos] = NULL;
            s->states[pos] = 2;
            s->size--;
            return;
        }
    }
}
void fusion_hss_clear(int h) {
    if (h <= 0 || h >= HS_POOL || !hss_pool[h].alive) return;
    HSetStr* s = &hss_pool[h];
    for (int i = 0; i < HS_CAP; i++) {
        if (s->data[i]) { free(s->data[i]); s->data[i] = NULL; }
    }
    memset(s->states, 0, sizeof(s->states));
    s->size = 0;
}

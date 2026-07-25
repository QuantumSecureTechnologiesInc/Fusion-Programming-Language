/* runtime/vector_runtime.c — Handle-based Vector runtime for Fusion.
 * Three vector types: VectorInt, VectorBool (stored as int), VectorString.
 * Pool capacity: 64 vectors each, dynamic capacity starting at 16.
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

#define VEC_POOL 64
#define VEC_INIT_CAP 16

/* --- VectorInt --- */
typedef struct {
    int* data;
    int len;
    int cap;
    int alive;
} VecInt;

static VecInt vi_pool[VEC_POOL];

int fusion_vi_create(void) {
    for (int i = 1; i < VEC_POOL; i++) {
        if (!vi_pool[i].alive) {
            vi_pool[i].data = (int*)malloc(VEC_INIT_CAP * sizeof(int));
            vi_pool[i].len = 0;
            vi_pool[i].cap = VEC_INIT_CAP;
            vi_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_vi_destroy(int h) {
    if (h > 0 && h < VEC_POOL && vi_pool[h].alive) {
        free(vi_pool[h].data);
        vi_pool[h].data = NULL;
        vi_pool[h].alive = 0;
    }
}
int fusion_vi_len(int h) {
    return (h > 0 && h < VEC_POOL && vi_pool[h].alive) ? vi_pool[h].len : 0;
}
int fusion_vi_is_empty(int h) {
    return fusion_vi_len(h) == 0;
}
void fusion_vi_push(int h, int value) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return;
    VecInt* v = &vi_pool[h];
    if (v->len >= v->cap) {
        int new_cap = v->cap * 2;
        int* nd = (int*)realloc(v->data, new_cap * sizeof(int));
        if (!nd) return;
        v->data = nd;
        v->cap = new_cap;
    }
    v->data[v->len++] = value;
}
int fusion_vi_get(int h, int index) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    if (index < 0 || index >= v->len) return 0;
    return v->data[index];
}
void fusion_vi_set(int h, int index, int value) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return;
    VecInt* v = &vi_pool[h];
    if (index < 0 || index >= v->len) return;
    v->data[index] = value;
}
int fusion_vi_pop(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    if (v->len == 0) return 0;
    return v->data[--v->len];
}
void fusion_vi_clear(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return;
    vi_pool[h].len = 0;
}
int fusion_vi_contains(int h, int value) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    for (int i = 0; i < v->len; i++) {
        if (v->data[i] == value) return 1;
    }
    return 0;
}
int fusion_vi_first(int h) {
    return fusion_vi_get(h, 0);
}
int fusion_vi_last(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    return (v->len > 0) ? v->data[v->len - 1] : 0;
}
int fusion_vi_sum(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    int total = 0;
    for (int i = 0; i < v->len; i++) total += v->data[i];
    return total;
}
int fusion_vi_min(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    if (v->len == 0) return 0;
    int m = v->data[0];
    for (int i = 1; i < v->len; i++) if (v->data[i] < m) m = v->data[i];
    return m;
}
int fusion_vi_max(int h) {
    if (h <= 0 || h >= VEC_POOL || !vi_pool[h].alive) return 0;
    VecInt* v = &vi_pool[h];
    if (v->len == 0) return 0;
    int m = v->data[0];
    for (int i = 1; i < v->len; i++) if (v->data[i] > m) m = v->data[i];
    return m;
}

/* --- VectorBool (stored as int: 0/1) --- */
typedef struct {
    int* data;
    int len;
    int cap;
    int alive;
} VecBool;

static VecBool vb_pool[VEC_POOL];

int fusion_vb_create(void) {
    for (int i = 1; i < VEC_POOL; i++) {
        if (!vb_pool[i].alive) {
            vb_pool[i].data = (int*)malloc(VEC_INIT_CAP * sizeof(int));
            vb_pool[i].len = 0;
            vb_pool[i].cap = VEC_INIT_CAP;
            vb_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_vb_destroy(int h) {
    if (h > 0 && h < VEC_POOL && vb_pool[h].alive) {
        free(vb_pool[h].data);
        vb_pool[h].data = NULL;
        vb_pool[h].alive = 0;
    }
}
int fusion_vb_len(int h) {
    return (h > 0 && h < VEC_POOL && vb_pool[h].alive) ? vb_pool[h].len : 0;
}
int fusion_vb_is_empty(int h) {
    return fusion_vb_len(h) == 0;
}
void fusion_vb_push(int h, int value) {
    if (h <= 0 || h >= VEC_POOL || !vb_pool[h].alive) return;
    VecBool* v = &vb_pool[h];
    if (v->len >= v->cap) {
        int new_cap = v->cap * 2;
        int* nd = (int*)realloc(v->data, new_cap * sizeof(int));
        if (!nd) return;
        v->data = nd;
        v->cap = new_cap;
    }
    v->data[v->len++] = (value != 0) ? 1 : 0;
}
int fusion_vb_get(int h, int index) {
    if (h <= 0 || h >= VEC_POOL || !vb_pool[h].alive) return 0;
    VecBool* v = &vb_pool[h];
    if (index < 0 || index >= v->len) return 0;
    return v->data[index];
}
void fusion_vb_pop(int h) {
    if (h <= 0 || h >= VEC_POOL || !vb_pool[h].alive) return;
    if (vb_pool[h].len > 0) vb_pool[h].len--;
}
void fusion_vb_clear(int h) {
    if (h <= 0 || h >= VEC_POOL || !vb_pool[h].alive) return;
    vb_pool[h].len = 0;
}

/* --- VectorString --- */
typedef struct {
    char** data;
    int len;
    int cap;
    int alive;
} VecStr;

static VecStr vs_pool[VEC_POOL];

int fusion_vs_create(void) {
    for (int i = 1; i < VEC_POOL; i++) {
        if (!vs_pool[i].alive) {
            vs_pool[i].data = (char**)malloc(VEC_INIT_CAP * sizeof(char*));
            vs_pool[i].len = 0;
            vs_pool[i].cap = VEC_INIT_CAP;
            vs_pool[i].alive = 1;
            return i;
        }
    }
    return 0;
}
void fusion_vs_destroy(int h) {
    if (h > 0 && h < VEC_POOL && vs_pool[h].alive) {
        for (int i = 0; i < vs_pool[h].len; i++) {
            if (vs_pool[h].data[i]) free(vs_pool[h].data[i]);
        }
        free(vs_pool[h].data);
        vs_pool[h].data = NULL;
        vs_pool[h].alive = 0;
    }
}
int fusion_vs_len(int h) {
    return (h > 0 && h < VEC_POOL && vs_pool[h].alive) ? vs_pool[h].len : 0;
}
int fusion_vs_is_empty(int h) {
    return fusion_vs_len(h) == 0;
}
void fusion_vs_push(int h, const char* value) {
    if (h <= 0 || h >= VEC_POOL || !vs_pool[h].alive) return;
    VecStr* v = &vs_pool[h];
    if (v->len >= v->cap) {
        int new_cap = v->cap * 2;
        char** nd = (char**)realloc(v->data, new_cap * sizeof(char*));
        if (!nd) return;
        v->data = nd;
        v->cap = new_cap;
    }
    v->data[v->len] = value ? _strdup(value) : NULL;
    v->len++;
}
const char* fusion_vs_get(int h, int index) {
    if (h <= 0 || h >= VEC_POOL || !vs_pool[h].alive) return "";
    VecStr* v = &vs_pool[h];
    if (index < 0 || index >= v->len) return "";
    return v->data[index] ? v->data[index] : "";
}
int fusion_vs_contains(int h, const char* value) {
    if (h <= 0 || h >= VEC_POOL || !vs_pool[h].alive) return 0;
    VecStr* v = &vs_pool[h];
    for (int i = 0; i < v->len; i++) {
        if (v->data[i] && value && strcmp(v->data[i], value) == 0) return 1;
    }
    return 0;
}
void fusion_vs_pop(int h) {
    if (h <= 0 || h >= VEC_POOL || !vs_pool[h].alive) return;
    VecStr* v = &vs_pool[h];
    if (v->len > 0) {
        v->len--;
        if (v->data[v->len]) { free(v->data[v->len]); v->data[v->len] = NULL; }
    }
}
void fusion_vs_clear(int h) {
    if (h <= 0 || h >= VEC_POOL || !vs_pool[h].alive) return;
    VecStr* v = &vs_pool[h];
    for (int i = 0; i < v->len; i++) {
        if (v->data[i]) { free(v->data[i]); v->data[i] = NULL; }
    }
    v->len = 0;
}

// Cross-platform Fusion runtime (POSIX + Win32).
// Original Linux/macOS implementation extended with Windows compatibility.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
#include <time.h>
#include <stdint.h>

#ifdef _WIN32
/* ── Windows includes ─────────────────────────────────────────────────────── */
/* winsock2.h MUST come before windows.h to avoid WinSock v1 conflicts */
#include <winsock2.h>
#include <ws2tcpip.h>
#include <io.h>
#include <direct.h>
#include <process.h>
#include <windows.h>
#pragma comment(lib, "ws2_32.lib")

#ifndef _CRT_SECURE_NO_WARNINGS
#define _CRT_SECURE_NO_WARNINGS
#endif

/* ssize_t is not standard on MSVC */
typedef intptr_t ssize_t;

/* getline() is not available on MSVC; implement it. */
static ssize_t win_getline(char** lineptr, size_t* n, FILE* stream) {
    if (!lineptr || !n || !stream) return -1;
    size_t cap = *n;
    char* buf = *lineptr;
    if (!buf) { cap = 128; buf = (char*)malloc(cap); if (!buf) return -1; }
    size_t len = 0;
    int ch;
    while ((ch = fgetc(stream)) != EOF) {
        if (len + 1 >= cap) {
            cap *= 2;
            char* tmp = (char*)realloc(buf, cap);
            if (!tmp) { *lineptr = buf; *n = cap; buf[len] = '\0'; return (ssize_t)len; }
            buf = tmp;
        }
        buf[len++] = (char)ch;
        if (ch == '\n') break;
    }
    if (len == 0 && ch == EOF) { free(buf); *lineptr = NULL; *n = 0; return -1; }
    buf[len] = '\0';
    *lineptr = buf;
    *n = cap;
    return (ssize_t)len;
}
#define getline win_getline

#define socket_close(fd) closesocket(fd)
#define plat_mkdir(path) _mkdir(path)
#define plat_strdup(s) _strdup(s)

static void wsa_ensure_init(void) {
    static int inited = 0;
    if (!inited) {
        WSADATA wsa;
        WSAStartup(MAKEWORD(2, 2), &wsa);
        inited = 1;
    }
}

#else
/* ── POSIX includes ───────────────────────────────────────────────────────── */
#include <unistd.h>
#include <pthread.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <arpa/inet.h>

#define socket_close(fd) close(fd)
#define plat_mkdir(path) mkdir(path, 0755)
#define plat_strdup(s) strdup(s)

static void wsa_ensure_init(void) { /* no-op on POSIX */ }

#ifndef _strdup
#define _strdup strdup
#endif

#endif /* _WIN32 */

void panic(const char* msg) {
    if (msg) {
        fprintf(stderr, "Runtime Panic: %s\n", msg);
    } else {
        fprintf(stderr, "Runtime Panic\n");
    }
    abort();
}

char* fusion_read_line(void) {
    char* line = NULL;
    size_t cap = 0;
    ssize_t n = getline(&line, &cap, stdin);
    if (n <= 0) {
        if (line) { line[0] = '\0'; return line; }
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    if (n > 0 && line[n - 1] == '\n') line[n - 1] = '\0';
    return line;
}

int string_starts_with(const char* s, const char* prefix) {
    if (!s || !prefix) return 0;
    size_t plen = strlen(prefix);
    return strncmp(s, prefix, plen) == 0;
}

static char* read_file_all(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) return NULL;
    if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return NULL; }
    long size = ftell(f);
    if (size < 0) { fclose(f); return NULL; }
    if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return NULL; }
    char* buf = (char*)malloc((size_t)size + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t n = fread(buf, 1, (size_t)size, f);
    fclose(f);
    buf[n] = '\0';
    return buf;
}

char* fusion_fs_read_to_string(const char* path) { return read_file_all(path); }

/* ── Compiler self-hosting support ─────────────────────────────────────────── */

/* Aliases the compiler externs expect (without _fs_ prefix) */
char* fusion_read_to_string(const char* path) {
    return read_file_all(path);
}

/* Returns non-zero if the content string indicates a read error (NULL or empty) */
int fusion_read_is_err(const char* content) {
    return (content == NULL || content[0] == '\0') ? 1 : 0;
}

/* POSIX access() wrapper — checks file existence/readability */
int access(const char* path, int mode) {
    if (!path) return -1;
#ifdef _WIN32
    return _access(path, mode);
#else
    return access(path, mode);
#endif
}

int fusion_fs_write_str(const char* path, const char* contents) {
    FILE* f = fopen(path, "wb");
    if (!f) return 0;
    if (contents) fwrite(contents, 1, strlen(contents), f);
    fclose(f);
    return 1;
}

int fusion_fs_append_str(const char* path, const char* contents) {
    FILE* f = fopen(path, "ab");
    if (!f) return 0;
    if (contents) fwrite(contents, 1, strlen(contents), f);
    fclose(f);
    return 1;
}

int fusion_fs_exists(const char* path) {
    struct stat st;
    return stat(path, &st) == 0;
}

int fusion_fs_create_dir(const char* path) {
    if (!path) return 0;
    return plat_mkdir(path) == 0 || errno == EEXIST;
}

int fusion_fs_remove_file(const char* path) {
    if (!path) return 0;
    return remove(path) == 0;
}

char* fusion_env_get(const char* key) {
    if (!key) return NULL;
    const char* val = getenv(key);
    if (!val) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    size_t len = strlen(val);
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    memcpy(out, val, len + 1);
    return out;
}

char* fusion_ipc_query(const char* key) {
    (void)key;
    char* empty = (char*)malloc(1);
    if (empty) empty[0] = '\0';
    return empty;
}

char* fusion_prompt(const char* prompt) {
    if (prompt) { fprintf(stdout, "%s", prompt); fflush(stdout); }
    return fusion_read_line();
}

/* ── argc / argv ──────────────────────────────────────────────────────────── */

#ifdef _WIN32
int fusion_argc(void) {
    extern int __argc;
    return __argc;
}
char* fusion_argv(int idx) {
    extern char** __argv;
    extern int __argc;
    if (idx < 0 || idx >= __argc) return NULL;
    return plat_strdup(__argv[idx]);
}
#else
int fusion_argc(void) {
    FILE* f = fopen("/proc/self/cmdline", "rb");
    if (!f) return 0;
    int count = 0, ch, last_null = 1;
    while ((ch = fgetc(f)) != EOF) {
        if (ch == '\0') { if (!last_null) { count++; last_null = 1; } }
        else { last_null = 0; }
    }
    fclose(f);
    return count;
}
char* fusion_argv(int idx) {
    FILE* f = fopen("/proc/self/cmdline", "rb");
    if (!f) return NULL;
    int current = 0;
    size_t cap = 256, len = 0;
    char* buf = (char*)malloc(cap);
    int ch;
    while ((ch = fgetc(f)) != EOF) {
        if (ch == '\0') {
            if (current == idx) { buf[len] = '\0'; fclose(f); return buf; }
            current++; len = 0; continue;
        }
        if (current == idx) {
            if (len + 1 >= cap) {
                cap *= 2;
                buf = (char*)realloc(buf, cap);
                if (!buf) { fclose(f); return NULL; }
            }
            buf[len++] = (char)ch;
        }
    }
    fclose(f);
    if (buf) free(buf);
    return NULL;
}
#endif

/* ── Time ─────────────────────────────────────────────────────────────────── */

#ifdef _WIN32
int fusion_time_now_ms(void) {
    FILETIME ft;
    GetSystemTimeAsFileTime(&ft);
    uint64_t t = ((uint64_t)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
    t = (t / 10000) - 11644473600000ULL;
    return (int)(t & 0x7fffffff);
}
void fusion_sleep_ms(int ms) {
    if (ms <= 0) return;
    Sleep((DWORD)ms);
}
#else
int fusion_time_now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    return (int)((ts.tv_sec * 1000) + (ts.tv_nsec / 1000000));
}
void fusion_sleep_ms(int ms) {
    if (ms <= 0) return;
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000;
    nanosleep(&ts, NULL);
}
#endif

/* ── RNG ──────────────────────────────────────────────────────────────────── */

static uint64_t g_rand_state = 88172645463325252ULL;
void fusion_rand_seed(int seed) {
    if (seed == 0) seed = 1;
    g_rand_state = ((uint64_t)seed << 32) ^ 0x9e3779b97f4a7c15ULL;
}
int fusion_rand_next(void) {
    uint64_t x = g_rand_state;
    x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
    g_rand_state = x;
    return (int)((x * 2685821657736338717ULL) & 0x7fffffff);
}

/* ── Hash ─────────────────────────────────────────────────────────────────── */

int fusion_hash32(const char* s) {
    if (!s) return 0;
    uint32_t h = 5381;
    int c;
    while ((c = *s++)) h = ((h << 5) + h) + (uint32_t)c;
    return (int)h;
}
int fusion_hmac32(const char* key, const char* msg) {
    return fusion_hash32(key) ^ fusion_hash32(msg);
}

/* ── Formatting ───────────────────────────────────────────────────────────── */

char* fusion_fmt_int(int v) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%d", v);
    return plat_strdup(buf);
}

char* fusion_fmt_pair(const char* k, const char* v) {
    if (!k) k = "";
    if (!v) v = "";
    size_t len = strlen(k) + strlen(v) + 2;
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    snprintf(out, len + 1, "%s=%s", k, v);
    return out;
}

char* fusion_str_concat(const char* a, const char* b) {
    if (!a) a = "";
    if (!b) b = "";
    size_t len = strlen(a) + strlen(b);
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    snprintf(out, len + 1, "%s%s", a, b);
    return out;
}

char* fusion_json_escape(const char* s) {
    if (!s) return plat_strdup("");
    size_t len = strlen(s);
    char* out = (char*)malloc(len * 2 + 1);
    if (!out) return NULL;
    size_t j = 0;
    for (size_t i = 0; i < len; i++) {
        char c = s[i];
        if (c == '"' || c == '\\') out[j++] = '\\';
        out[j++] = c;
    }
    out[j] = '\0';
    return out;
}

char* fusion_json_kv_string(const char* k, const char* v) {
    char* ek = fusion_json_escape(k);
    char* ev = fusion_json_escape(v);
    size_t len = strlen(ek) + strlen(ev) + 6;
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    snprintf(out, len + 1, "\"%s\":\"%s\"", ek, ev);
    free(ek); free(ev);
    return out;
}

char* fusion_json_kv_int(const char* k, int v) {
    char* ek = fusion_json_escape(k);
    char buf[64];
    snprintf(buf, sizeof(buf), "%d", v);
    size_t len = strlen(ek) + strlen(buf) + 4;
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    snprintf(out, len + 1, "\"%s\":%s", ek, buf);
    free(ek);
    return out;
}

/* ── Networking ───────────────────────────────────────────────────────────── */

int fusion_tcp_connect(const char* host, int port) {
    wsa_ensure_init();
    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", port);
    struct addrinfo hints, *res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    if (getaddrinfo(host, port_str, &hints, &res) != 0) return -1;
    int sock = -1;
    for (struct addrinfo* p = res; p; p = p->ai_next) {
        sock = (int)socket(p->ai_family, p->ai_socktype, p->ai_protocol);
        if (sock < 0) continue;
        if (connect(sock, p->ai_addr, (int)p->ai_addrlen) == 0) break;
        socket_close(sock);
        sock = -1;
    }
    freeaddrinfo(res);
    return sock;
}

int fusion_tcp_send_str(int fd, const char* data) {
    if (fd < 0 || !data) return -1;
    return (int)send(fd, data, (int)strlen(data), 0);
}

char* fusion_tcp_recv_str(int fd, int max_bytes) {
    if (fd < 0 || max_bytes <= 0) return plat_strdup("");
    char* buf = (char*)malloc((size_t)max_bytes + 1);
    if (!buf) return NULL;
    int n = (int)recv(fd, buf, (size_t)max_bytes, 0);
    if (n < 0) n = 0;
    buf[n] = '\0';
    return buf;
}

void fusion_tcp_close(int fd) {
    if (fd >= 0) socket_close(fd);
}

int fusion_udp_send_str(const char* host, int port, const char* data) {
    wsa_ensure_init();
    if (!host || !data) return -1;
    int sock = (int)socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) return -1;
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((uint16_t)port);
    if (inet_pton(AF_INET, host, &addr.sin_addr) <= 0) { socket_close(sock); return -1; }
    int n = (int)sendto(sock, data, (int)strlen(data), 0, (struct sockaddr*)&addr, (int)sizeof(addr));
    socket_close(sock);
    return n;
}

char* fusion_udp_recv_str(int port, int max_bytes) {
    wsa_ensure_init();
    if (max_bytes <= 0) return plat_strdup("");
    int sock = (int)socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) return plat_strdup("");
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons((uint16_t)port);
    if (bind(sock, (struct sockaddr*)&addr, (int)sizeof(addr)) != 0) { socket_close(sock); return plat_strdup(""); }
    char* buf = (char*)malloc((size_t)max_bytes + 1);
    if (!buf) { socket_close(sock); return plat_strdup(""); }
    int n = (int)recvfrom(sock, buf, (size_t)max_bytes, 0, NULL, NULL);
    if (n < 0) n = 0;
    buf[n] = '\0';
    socket_close(sock);
    return buf;
}

/* ── Mutex ────────────────────────────────────────────────────────────────── */

#ifdef _WIN32
int fusion_mutex_new(void) {
    CRITICAL_SECTION* cs = (CRITICAL_SECTION*)malloc(sizeof(CRITICAL_SECTION));
    if (!cs) return 0;
    InitializeCriticalSection(cs);
    return (int)(intptr_t)cs;
}
void fusion_mutex_lock(int handle) {
    if (handle == 0) return;
    EnterCriticalSection((CRITICAL_SECTION*)(intptr_t)handle);
}
void fusion_mutex_unlock(int handle) {
    if (handle == 0) return;
    LeaveCriticalSection((CRITICAL_SECTION*)(intptr_t)handle);
}
void fusion_mutex_free(int handle) {
    if (handle == 0) return;
    CRITICAL_SECTION* cs = (CRITICAL_SECTION*)(intptr_t)handle;
    DeleteCriticalSection(cs);
    free(cs);
}
#else
int fusion_mutex_new(void) {
    pthread_mutex_t* m = (pthread_mutex_t*)malloc(sizeof(pthread_mutex_t));
    if (!m) return 0;
    pthread_mutex_init(m, NULL);
    return (int)(intptr_t)m;
}
void fusion_mutex_lock(int handle) {
    if (handle == 0) return;
    pthread_mutex_lock((pthread_mutex_t*)(intptr_t)handle);
}
void fusion_mutex_unlock(int handle) {
    if (handle == 0) return;
    pthread_mutex_unlock((pthread_mutex_t*)(intptr_t)handle);
}
void fusion_mutex_free(int handle) {
    if (handle == 0) return;
    pthread_mutex_t* m = (pthread_mutex_t*)(intptr_t)handle;
    pthread_mutex_destroy(m);
    free(m);
}
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

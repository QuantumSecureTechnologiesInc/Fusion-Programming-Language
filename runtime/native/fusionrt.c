#include "fusionrt.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdarg.h>
#include <stdint.h>

#ifdef _WIN32
#include <direct.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")

#define mkdir(path, mode) _mkdir(path)
#define PATH_SEP "\\"
#define FUSION_CLOSE_SOCKET(fd) closesocket((SOCKET)(intptr_t)(fd))
#else
#include <sys/stat.h>
#include <sys/time.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <pthread.h>

#define PATH_SEP "/"
#define FUSION_CLOSE_SOCKET(fd) close((int)(fd))
#endif

// ========================================
// String Pool for Interned Returns
// ========================================
#define STRING_POOL_SIZE (16 * 1024 * 1024) // 16MB
static char string_pool[STRING_POOL_SIZE];
static size_t pool_offset = 0;

static const char *pool_intern(const char *s) {
  if (!s)
    return pool_intern("");

  size_t len = strlen(s) + 1;
  if (pool_offset + len > STRING_POOL_SIZE) {
    fprintf(stderr, "FATAL: Fusion string pool exhausted\n");
    exit(1);
  }

  char *dest = string_pool + pool_offset;
  memcpy(dest, s, len);
  pool_offset += len;
  return dest;
}

static const char *pool_sprintf(const char *fmt, ...) {
  char buf[4096];
  va_list args;
  va_start(args, fmt);
  vsnprintf(buf, sizeof(buf), fmt, args);
  va_end(args);
  return pool_intern(buf);
}

// ========================================
// Global State
// ========================================
static int g_argc = 0;
static char **g_argv = NULL;
static uint64_t g_rand_state = 12345;

void fusion_rt_init(void) {
  // Called before any runtime functions
  pool_offset = 0;
  g_rand_state = (uint64_t)time(NULL);
}

void fusion_rt_shutdown(void) {
  // Cleanup (string pool is static, nothing to free)
}

// ========================================
// I/O Functions
// ========================================
const char *fusion_read_line(void) {
  char buf[8192];
  if (fgets(buf, sizeof(buf), stdin)) {
    // Remove trailing newline
    size_t len = strlen(buf);
    if (len > 0 && buf[len - 1] == '\n')
      buf[len - 1] = '\0';
    return pool_intern(buf);
  }
  return pool_intern("");
}

// ========================================
// File System
// ========================================
const char *fusion_fs_read_to_string(const char *path) {
  FILE *f = fopen(path, "rb");
  if (!f)
    return pool_intern("");

  fseek(f, 0, SEEK_END);
  long size = ftell(f);
  fseek(f, 0, SEEK_SET);

  if (size < 0 || size > STRING_POOL_SIZE - pool_offset - 1) {
    fclose(f);
    return pool_intern("");
  }

  char *dest = string_pool + pool_offset;
  size_t read = fread(dest, 1, size, f);
  fclose(f);

  dest[read] = '\0';
  pool_offset += read + 1;
  return dest;
}

bool fusion_fs_write_str(const char *path, const char *contents) {
  FILE *f = fopen(path, "wb");
  if (!f)
    return false;

  size_t len = strlen(contents);
  size_t written = fwrite(contents, 1, len, f);
  fclose(f);
  return written == len;
}

bool fusion_fs_append_str(const char *path, const char *contents) {
  FILE *f = fopen(path, "ab");
  if (!f)
    return false;

  size_t len = strlen(contents);
  size_t written = fwrite(contents, 1, len, f);
  fclose(f);
  return written == len;
}

bool fusion_fs_exists(const char *path) {
  FILE *f = fopen(path, "rb");
  if (f) {
    fclose(f);
    return true;
  }
  return false;
}

bool fusion_fs_create_dir(const char *path) { return mkdir(path, 0755) == 0; }

bool fusion_fs_remove_file(const char *path) { return remove(path) == 0; }

// ========================================
// Path Manipulation
// ========================================
const char *fusion_path_join(const char *a, const char *b) {
  return pool_sprintf("%s%s%s", a, PATH_SEP, b);
}

const char *fusion_path_basename(const char *p) {
  const char *last_sep = strrchr(p, PATH_SEP[0]);
  if (last_sep)
    return pool_intern(last_sep + 1);
  return pool_intern(p);
}

const char *fusion_path_dirname(const char *p) {
  const char *last_sep = strrchr(p, PATH_SEP[0]);
  if (!last_sep)
    return pool_intern(".");

  size_t len = last_sep - p;
  if (len == 0)
    return pool_intern(PATH_SEP);

  char buf[4096];
  if (len >= sizeof(buf))
    len = sizeof(buf) - 1;
  memcpy(buf, p, len);
  buf[len] = '\0';
  return pool_intern(buf);
}

// ========================================
// Environment
// ========================================
const char *fusion_env_get(const char *key) {
  char *val = getenv(key);
  return val ? pool_intern(val) : pool_intern("");
}

int64_t fusion_argc(void) { return g_argc; }

const char *fusion_argv(int64_t idx) {
  if (idx < 0 || idx >= g_argc)
    return pool_intern("");
  return pool_intern(g_argv[idx]);
}

// ========================================
// Time
// ========================================
int64_t fusion_time_now_ms(void) {
#ifdef _WIN32
  return (int64_t)GetTickCount64();
#else
  struct timeval tv;
  gettimeofday(&tv, NULL);
  return (int64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;
#endif
}

void fusion_sleep_ms(int64_t ms) {
#ifdef _WIN32
  Sleep((DWORD)ms);
#else
  usleep(ms * 1000);
#endif
}

// ========================================
// Random (simple LCG)
// ========================================
void fusion_rand_seed(int64_t seed) { g_rand_state = (uint64_t)seed; }

int64_t fusion_rand_next(void) {
  g_rand_state = g_rand_state * 6364136223846793005ULL + 1442695040888963407ULL;
  return (int64_t)(g_rand_state >> 32);
}

// ========================================
// Crypto/Hash (simple DJB2)
// ========================================
int64_t fusion_hash32(const char *s) {
  uint64_t hash = 5381;
  int c;
  while ((c = *s++)) {
    hash = ((hash << 5) + hash) + c;
  }
  return (int64_t)(hash & 0xFFFFFFFF);
}

int64_t fusion_hmac32(const char *key, const char *msg) {
  // Simplified: just hash(key + msg)
  char buf[8192];
  snprintf(buf, sizeof(buf), "%s%s", key, msg);
  return fusion_hash32(buf);
}

// ========================================
// JSON/Formatting
// ========================================
const char *fusion_fmt_int(int64_t v) {
  return pool_sprintf("%lld", (long long)v);
}

const char *fusion_fmt_pair(const char *k, const char *v) {
  return pool_sprintf("%s=%s", k, v);
}

const char *fusion_str_concat(const char *a, const char *b) {
  if (!a) a = "";
  if (!b) b = "";
  return pool_sprintf("%s%s", a, b);
}

const char *fusion_json_escape(const char *s) {
  char buf[8192];
  char *dst = buf;
  const char *src = s;

  while (*src && (dst - buf) < sizeof(buf) - 10) {
    switch (*src) {
    case '"':
      *dst++ = '\\';
      *dst++ = '"';
      break;
    case '\\':
      *dst++ = '\\';
      *dst++ = '\\';
      break;
    case '\n':
      *dst++ = '\\';
      *dst++ = 'n';
      break;
    case '\r':
      *dst++ = '\\';
      *dst++ = 'r';
      break;
    case '\t':
      *dst++ = '\\';
      *dst++ = 't';
      break;
    default:
      *dst++ = *src;
      break;
    }
    src++;
  }
  *dst = '\0';
  return pool_intern(buf);
}

const char *fusion_json_kv_string(const char *k, const char *v) {
  return pool_sprintf("\"%s\":\"%s\"", fusion_json_escape(k),
                      fusion_json_escape(v));
}

const char *fusion_json_kv_int(const char *k, int64_t v) {
  return pool_sprintf("\"%s\":%lld", fusion_json_escape(k), (long long)v);
}

// ========================================
// Networking (TCP/UDP)
// ========================================
static int fusion_ws_init = 0;

static void fusion_ensure_ws(void) {
#ifdef _WIN32
  if (!fusion_ws_init) {
    WSADATA wsa;
    WSAStartup(MAKEWORD(2, 2), &wsa);
    fusion_ws_init = 1;
  }
#endif
}

static int64_t fusion_parse_host_port(const char* host, int64_t port,
                                       struct sockaddr_in* addr) {
  memset(addr, 0, sizeof(*addr));
  addr->sin_family = AF_INET;
  addr->sin_port = htons((uint16_t)port);
  if (!host || host[0] == '\0' || strcmp(host, "0.0.0.0") == 0) {
    addr->sin_addr.s_addr = INADDR_ANY;
  } else {
    struct addrinfo hints, *res = NULL;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    if (getaddrinfo(host, NULL, &hints, &res) != 0 || !res) {
      addr->sin_addr.s_addr = inet_addr(host);
      if (addr->sin_addr.s_addr == INADDR_NONE) return -1;
    } else {
      struct sockaddr_in* r = (struct sockaddr_in*)res->ai_addr;
      addr->sin_addr = r->sin_addr;
      freeaddrinfo(res);
    }
  }
  return 0;
}

int64_t fusion_tcp_connect(const char *host, int64_t port) {
  fusion_ensure_ws();
  struct sockaddr_in addr;
  if (fusion_parse_host_port(host, port, &addr) < 0) return -1;
  SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
  if (s == INVALID_SOCKET) return -1;
  if (connect(s, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
    FUSION_CLOSE_SOCKET((int64_t)(intptr_t)s);
    return -1;
  }
  return (int64_t)(intptr_t)s;
}

int64_t fusion_tcp_send_str(int64_t fd, const char *data) {
  if (!data) return -1;
  int64_t len = (int64_t)strlen(data);
  return (int64_t)send((SOCKET)(intptr_t)fd, data, (int)len, 0);
}

const char *fusion_tcp_recv_str(int64_t fd, int64_t max_bytes) {
  if (max_bytes <= 0) max_bytes = 65536;
  char* buf = (char*)malloc((size_t)max_bytes + 1);
  if (!buf) return pool_intern("");
  int64_t n = (int64_t)recv((SOCKET)(intptr_t)fd, buf, (int)max_bytes, 0);
  if (n <= 0) { free(buf); return pool_intern(""); }
  buf[n] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

void fusion_tcp_close(int64_t fd) {
  if (fd > 0) FUSION_CLOSE_SOCKET((int64_t)(intptr_t)fd);
}

int64_t fusion_udp_send_str(const char *host, int64_t port, const char *data) {
  fusion_ensure_ws();
  struct sockaddr_in addr;
  if (fusion_parse_host_port(host, port, &addr) < 0) return -1;
  SOCKET s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
  if (s == INVALID_SOCKET) return -1;
  int64_t len = (int64_t)strlen(data);
  int64_t sent = (int64_t)sendto(s, data, (int)len, 0,
                                   (struct sockaddr*)&addr, sizeof(addr));
  FUSION_CLOSE_SOCKET((int64_t)(intptr_t)s);
  return sent;
}

const char *fusion_udp_recv_str(int64_t port, int64_t max_bytes) {
  fusion_ensure_ws();
  if (max_bytes <= 0) max_bytes = 65536;
  SOCKET s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
  if (s == INVALID_SOCKET) return pool_intern("");
  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_port = htons((uint16_t)port);
  addr.sin_addr.s_addr = INADDR_ANY;
  if (bind(s, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
    FUSION_CLOSE_SOCKET((int64_t)(intptr_t)s);
    return pool_intern("");
  }
  char* buf = (char*)malloc((size_t)max_bytes + 1);
  if (!buf) { FUSION_CLOSE_SOCKET((int64_t)(intptr_t)s); return pool_intern(""); }
  int64_t n = (int64_t)recvfrom(s, buf, (int)max_bytes, 0, NULL, NULL);
  FUSION_CLOSE_SOCKET((int64_t)(intptr_t)s);
  if (n <= 0) { free(buf); return pool_intern(""); }
  buf[n] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

// ========================================
// Synchronization
// ========================================
#ifdef _WIN32
int64_t fusion_mutex_new(void) {
  CRITICAL_SECTION* cs = (CRITICAL_SECTION*)malloc(sizeof(CRITICAL_SECTION));
  if (!cs) return 0;
  InitializeCriticalSection(cs);
  return (int64_t)(intptr_t)cs;
}

void fusion_mutex_lock(int64_t handle) {
  if (handle) EnterCriticalSection((CRITICAL_SECTION*)(intptr_t)handle);
}

void fusion_mutex_unlock(int64_t handle) {
  if (handle) LeaveCriticalSection((CRITICAL_SECTION*)(intptr_t)handle);
}

void fusion_mutex_free(int64_t handle) {
  if (handle) {
    CRITICAL_SECTION* cs = (CRITICAL_SECTION*)(intptr_t)handle;
    DeleteCriticalSection(cs);
    free(cs);
  }
}
#else
int64_t fusion_mutex_new(void) {
  pthread_mutex_t* mtx = (pthread_mutex_t*)malloc(sizeof(pthread_mutex_t));
  if (!mtx) return 0;
  pthread_mutex_init(mtx, NULL);
  return (int64_t)(intptr_t)mtx;
}

void fusion_mutex_lock(int64_t handle) {
  if (handle) pthread_mutex_lock((pthread_mutex_t*)(intptr_t)handle);
}

void fusion_mutex_unlock(int64_t handle) {
  if (handle) pthread_mutex_unlock((pthread_mutex_t*)(intptr_t)handle);
}

void fusion_mutex_free(int64_t handle) {
  if (handle) {
    pthread_mutex_t* mtx = (pthread_mutex_t*)(intptr_t)handle;
    pthread_mutex_destroy(mtx);
    free(mtx);
  }
}
#endif

// ========================================
// Memory Management
// ========================================
int64_t fusion_malloc(int64_t size) {
  if (size <= 0) return 0;
  void* p = malloc((size_t)size);
  if (!p) {
    fprintf(stderr, "FATAL: fusion_malloc(%lld) failed\n", (long long)size);
    exit(1);
  }
  memset(p, 0, (size_t)size);
  return (int64_t)(intptr_t)p;
}

void fusion_free(int64_t ptr) {
  if (ptr != 0) {
    free((void*)(intptr_t)ptr);
  }
}

int64_t fusion_realloc(int64_t ptr, int64_t size) {
  if (size <= 0) return 0;
  void* p;
  if (ptr == 0) {
    p = malloc((size_t)size);
  } else {
    p = realloc((void*)(intptr_t)ptr, (size_t)size);
  }
  if (!p) {
    fprintf(stderr, "FATAL: fusion_realloc(%lld) failed\n", (long long)size);
    exit(1);
  }
  return (int64_t)(intptr_t)p;
}

void fusion_memcpy(int64_t dest, int64_t src, int64_t n) {
  if (dest && src && n > 0) {
    memcpy((void*)(intptr_t)dest, (const void*)(intptr_t)src, (size_t)n);
  }
}

void fusion_memset(int64_t dest, int64_t val, int64_t n) {
  if (dest && n > 0) {
    memset((void*)(intptr_t)dest, (int)val, (size_t)n);
  }
}

// ========================================
// String Operations
// ========================================
int64_t fusion_strlen(const char* s) {
  if (!s) return 0;
  return (int64_t)strlen(s);
}

int64_t fusion_strcmp(const char* a, const char* b) {
  if (!a && !b) return 0;
  if (!a) return -1;
  if (!b) return 1;
  return (int64_t)strcmp(a, b);
}

int64_t fusion_strcpy(const char* src) {
  if (!src) return 0;
  size_t len = strlen(src) + 1;
  char* dest = (char*)malloc(len);
  if (!dest) return 0;
  memcpy(dest, src, len);
  return (int64_t)(intptr_t)dest;
}

const char* fusion_strdup(const char* s) {
  if (!s) return pool_intern("");
  return pool_intern(s);
}

// ========================================
// Console I/O
// ========================================
void fusion_puts(const char* s) {
  if (s) puts(s);
}

void fusion_print(const char* s) {
  if (s) fputs(s, stdout);
}

void fusion_println(const char* s) {
  if (s) {
    fputs(s, stdout);
    fputc('\n', stdout);
  } else {
    fputc('\n', stdout);
  }
}

void fusion_print_int(int64_t v) {
  printf("%lld\n", (long long)v);
}

void fusion_exit(int64_t code) {
  exit((int)code);
}

// ========================================
// String Utility Functions
// ========================================
int64_t fusion_ptr_read_byte(int64_t ptr, int64_t offset) {
  if (!ptr) return 0;
  const unsigned char* p = (const unsigned char*)(intptr_t)ptr;
  return (int64_t)p[offset];
}

void fusion_ptr_write_byte(int64_t ptr, int64_t offset, int64_t val) {
  if (!ptr) return;
  unsigned char* p = (unsigned char*)(intptr_t)ptr;
  p[offset] = (unsigned char)val;
}

int64_t fusion_ptr_read_int(int64_t ptr, int64_t index) {
  if (!ptr) return 0;
  const int64_t* p = (const int64_t*)(intptr_t)ptr;
  return p[index];
}

void fusion_ptr_write_int(int64_t ptr, int64_t index, int64_t val) {
  if (!ptr) return;
  int64_t* p = (int64_t*)(intptr_t)ptr;
  p[index] = val;
}

const char* fusion_str_repeat(int64_t ch, int64_t count) {
  if (count <= 0 || count > 65536) return pool_intern("");
  char* buf = (char*)malloc((size_t)count + 1);
  if (!buf) return pool_intern("");
  memset(buf, (int)ch, (size_t)count);
  buf[count] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

const char* fusion_str_trim(const char* s) {
  if (!s) return pool_intern("");
  const char* start = s;
  while (*start == ' ' || *start == '\t' || *start == '\n' || *start == '\r') start++;
  const char* end = s + strlen(s);
  while (end > start && (*(end-1) == ' ' || *(end-1) == '\t' || *(end-1) == '\n' || *(end-1) == '\r')) end--;
  size_t len = (size_t)(end - start);
  char* buf = (char*)malloc(len + 1);
  if (!buf) return pool_intern("");
  memcpy(buf, start, len);
  buf[len] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

const char* fusion_str_substring(const char* s, int64_t start, int64_t end_pos) {
  if (!s) return pool_intern("");
  int64_t len = (int64_t)strlen(s);
  if (start < 0) start = 0;
  if (end_pos > len) end_pos = len;
  if (start >= end_pos) return pool_intern("");
  size_t slen = (size_t)(end_pos - start);
  char* buf = (char*)malloc(slen + 1);
  if (!buf) return pool_intern("");
  memcpy(buf, s + start, slen);
  buf[slen] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

const char* fusion_int_to_string(int64_t n) {
  return pool_sprintf("%lld", (long long)n);
}

int64_t fusion_string_to_int(const char* s) {
  if (!s || s[0] == '\0') return 0;
  int64_t result = 0;
  int sign = 1;
  const char* p = s;
  if (*p == '-') { sign = -1; p++; }
  else if (*p == '+') { p++; }
  while (*p >= '0' && *p <= '9') {
    result = result * 10 + (*p - '0');
    p++;
  }
  return result * sign;
}

bool fusion_str_starts_with(const char* s, const char* prefix) {
  if (!s || !prefix) return false;
  size_t plen = strlen(prefix);
  return strncmp(s, prefix, plen) == 0;
}

bool fusion_str_ends_with(const char* s, const char* suffix) {
  if (!s || !suffix) return false;
  size_t slen = strlen(s);
  size_t xlen = strlen(suffix);
  if (xlen > slen) return false;
  return strcmp(s + slen - xlen, suffix) == 0;
}

const char* fusion_str_replace(const char* s, const char* from, const char* to_str) {
  if (!s || !from || !to_str) return pool_intern("");
  const char* pos = strstr(s, from);
  if (!pos) return pool_intern(s);
  size_t prefix_len = (size_t)(pos - s);
  size_t from_len = strlen(from);
  size_t to_len = strlen(to_str);
  size_t suffix_len = strlen(pos + from_len);
  size_t total = prefix_len + to_len + suffix_len;
  char* buf = (char*)malloc(total + 1);
  if (!buf) return pool_intern(s);
  memcpy(buf, s, prefix_len);
  memcpy(buf + prefix_len, to_str, to_len);
  memcpy(buf + prefix_len + to_len, pos + from_len, suffix_len);
  buf[total] = '\0';
  const char* result = pool_intern(buf);
  free(buf);
  return result;
}

// ========================================
// Runtime Entry Point Shim
// ========================================
// This allows C runtime to capture argc/argv before Fusion main
void fusion_rt_set_args(int argc, char **argv) {
  g_argc = argc;
  g_argv = argv;
}

// ========================================
// Concatenation Helpers (3 and 4 args)
// ========================================
const char* fusion_str_concat3(const char* a, const char* b, const char* c) {
  if (!a) a = "";
  if (!b) b = "";
  if (!c) c = "";
  return pool_sprintf("%s%s%s", a, b, c);
}

const char* fusion_str_concat4(const char* a, const char* b, const char* c, const char* d) {
  if (!a) a = "";
  if (!b) b = "";
  if (!c) c = "";
  if (!d) d = "";
  return pool_sprintf("%s%s%s%s", a, b, c, d);
}

// ========================================
// Memory Aliases (stdlib/memory.fu compatibility)
// ========================================
int64_t fu_malloc(int64_t size_bytes) {
  return fusion_malloc(size_bytes);
}

void fu_free(int64_t ptr) {
  fusion_free(ptr);
}

#ifndef FUSIONRT_H
#define FUSIONRT_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// ========================================
// Core String Pool
// ========================================
// NOTE: All string returns are pooled/interned.
// Caller does NOT free. Valid until fusion_rt_shutdown().

// ========================================
// I/O Functions
// ========================================
const char* fusion_read_line(void);

// ========================================
// File System
// ========================================
const char* fusion_fs_read_to_string(const char* path);
bool fusion_fs_write_str(const char* path, const char* contents);
bool fusion_fs_append_str(const char* path, const char* contents);
bool fusion_fs_exists(const char* path);
bool fusion_fs_create_dir(const char* path);
bool fusion_fs_remove_file(const char* path);

// ========================================
// Path Manipulation
// ========================================
const char* fusion_path_join(const char* a, const char* b);
const char* fusion_path_basename(const char* p);
const char* fusion_path_dirname(const char* p);

// ========================================
// Environment
// ========================================
const char* fusion_env_get(const char* key);
int64_t fusion_argc(void);
const char* fusion_argv(int64_t idx);

// ========================================
// Time
// ========================================
int64_t fusion_time_now_ms(void);
void fusion_sleep_ms(int64_t ms);

// ========================================
// Random
// ========================================
void fusion_rand_seed(int64_t seed);
int64_t fusion_rand_next(void);

// ========================================
// Crypto/Hash
// ========================================
int64_t fusion_hash32(const char* s);
int64_t fusion_hmac32(const char* key, const char* msg);

// ========================================
// JSON/Formatting
// ========================================
const char* fusion_fmt_int(int64_t v);
const char* fusion_fmt_pair(const char* k, const char* v);
const char* fusion_json_escape(const char* s);
const char* fusion_json_kv_string(const char* k, const char* v);
const char* fusion_json_kv_int(const char* k, int64_t v);
const char* fusion_str_concat(const char* a, const char* b);

// ========================================
// Networking (TCP)
// ========================================
int64_t fusion_tcp_connect(const char* host, int64_t port);
int64_t fusion_tcp_send_str(int64_t fd, const char* data);
const char* fusion_tcp_recv_str(int64_t fd, int64_t max_bytes);
void fusion_tcp_close(int64_t fd);

// ========================================
// Networking (UDP)
// ========================================
int64_t fusion_udp_send_str(const char* host, int64_t port, const char* data);
const char* fusion_udp_recv_str(int64_t port, int64_t max_bytes);

// ========================================
// Synchronization
// ========================================
int64_t fusion_mutex_new(void);
void fusion_mutex_lock(int64_t handle);
void fusion_mutex_unlock(int64_t handle);
void fusion_mutex_free(int64_t handle);

// ========================================
// Memory Management
// ========================================
int64_t fusion_malloc(int64_t size);
void fusion_free(int64_t ptr);
int64_t fusion_realloc(int64_t ptr, int64_t size);
void fusion_memcpy(int64_t dest, int64_t src, int64_t n);
void fusion_memset(int64_t dest, int64_t val, int64_t n);

// ========================================
// String Operations
// ========================================
int64_t fusion_strlen(const char* s);
int64_t fusion_strcmp(const char* a, const char* b);
int64_t fusion_strcpy(const char* src);
const char* fusion_strdup(const char* s);

// ========================================
// Console I/O
// ========================================
void fusion_puts(const char* s);
void fusion_print(const char* s);
void fusion_println(const char* s);
void fusion_print_int(int64_t v);
void fusion_exit(int64_t code);

// ========================================
// Pointer Helpers
// ========================================
int64_t fusion_ptr_read_byte(int64_t ptr, int64_t offset);
void fusion_ptr_write_byte(int64_t ptr, int64_t offset, int64_t val);
int64_t fusion_ptr_read_int(int64_t ptr, int64_t index);
void fusion_ptr_write_int(int64_t ptr, int64_t index, int64_t val);

// ========================================
// String Utilities
// ========================================
const char* fusion_str_repeat(int64_t ch, int64_t count);
const char* fusion_str_trim(const char* s);
const char* fusion_str_substring(const char* s, int64_t start, int64_t end_pos);
const char* fusion_int_to_string(int64_t n);
int64_t fusion_string_to_int(const char* s);
bool fusion_str_starts_with(const char* s, const char* prefix);
bool fusion_str_ends_with(const char* s, const char* suffix);
const char* fusion_str_replace(const char* s, const char* from, const char* to_str);

// ========================================
// Concatenation Helpers
// ========================================
const char* fusion_str_concat3(const char* a, const char* b, const char* c);
const char* fusion_str_concat4(const char* a, const char* b, const char* c, const char* d);

// ========================================
// Memory Aliases (stdlib/memory.fu compatibility)
// ========================================
int64_t fu_malloc(int64_t size_bytes);
void fu_free(int64_t ptr);

// ========================================
// Runtime Lifecycle
// ========================================
void fusion_rt_init(void);
void fusion_rt_shutdown(void);
void fusion_rt_set_args(int argc, char** argv);

#ifdef __cplusplus
}
#endif

#endif // FUSIONRT_H

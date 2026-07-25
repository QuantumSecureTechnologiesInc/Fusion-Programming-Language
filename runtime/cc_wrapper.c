/* cc_wrapper.c — wraps clang, injecting collection runtime .o on link steps.
 * Uses CreateProcessA for reliable path-with-spaces handling on Windows.
 * Injects: hashmap_runtime.o, vector_runtime.o, hashset_runtime.o
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#define PATH_SEP '\\'
#define MAX_RT 4

int main(int argc, char* argv[]) {
    char self[4096];
    GetModuleFileNameA(NULL, self, sizeof(self));
    char* sep = strrchr(self, PATH_SEP);
    if (!sep) sep = strrchr(self, '/');
    if (sep) *(sep + 1) = '\0';

    char clang_path[4096];
    snprintf(clang_path, sizeof(clang_path), "%sclang_real.exe", self);

    char rt_objs[MAX_RT][4096];
    const char* rt_names[MAX_RT] = {
        "runtime.o", "hashmap_runtime.o", "vector_runtime.o", "hashset_runtime.o"
    };
    for (int i = 0; i < MAX_RT; i++) {
        char raw[4096];
        snprintf(raw, sizeof(raw), "%s..%cruntime%c%s", self, PATH_SEP, PATH_SEP, rt_names[i]);
        GetFullPathNameA(raw, sizeof(rt_objs[i]), rt_objs[i], NULL);
    }

    /* Check if this is a link step (no -c flag) */
    int is_link = 1;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-c") == 0) { is_link = 0; break; }
    }

    /* Build full command line with proper quoting for spaces */
    char cmdline[32768];
    int pos = 0;
    pos += snprintf(cmdline + pos, sizeof(cmdline) - pos, "\"%s\"", clang_path);
    for (int i = 1; i < argc; i++) {
        pos += snprintf(cmdline + pos, sizeof(cmdline) - pos, " \"%s\"", argv[i]);
    }
    if (is_link) {
        for (int i = 0; i < MAX_RT; i++) {
            pos += snprintf(cmdline + pos, sizeof(cmdline) - pos, " \"%s\"", rt_objs[i]);
        }
    }

    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    ZeroMemory(&pi, sizeof(pi));

    if (!CreateProcessA(clang_path, cmdline, NULL, NULL, FALSE,
                        0, NULL, NULL, &si, &pi)) {
        fprintf(stderr, "cc_wrapper: CreateProcess failed (%lu)\n", GetLastError());
        return 1;
    }
    WaitForSingleObject(pi.hProcess, INFINITE);
    DWORD exit_code = 0;
    GetExitCodeProcess(pi.hProcess, &exit_code);
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    return (int)exit_code;
}

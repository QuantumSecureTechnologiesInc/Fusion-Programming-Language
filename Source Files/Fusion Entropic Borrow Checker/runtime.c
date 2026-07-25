#include <stdio.h>
#include <stdlib.h>

void panic(char* message) {
    fprintf(stderr, "Runtime Panic: %s\n", message);
    abort();
}

void log_collision(long long target_id, long long existing_kind, long long new_kind) {
    fprintf(stderr,
        "Entropy Collision: target=%lld existing_kind=%lld new_kind=%lld\n",
        target_id,
        existing_kind,
        new_kind
    );
}

void log_collision_detail(long long target_id,
                          long long existing_kind,
                          long long new_kind,
                          long long existing_id,
                          long long new_id) {
    const char *existing_label = existing_kind == 1 ? "mutable" : "immutable";
    const char *new_label = new_kind == 1 ? "mutable" : "immutable";
    fprintf(stderr,
        "Entropy Collision: target=%lld existing=%s(id=%lld) new=%s(id=%lld)\n",
        target_id,
        existing_label,
        existing_id,
        new_label,
        new_id
    );
}

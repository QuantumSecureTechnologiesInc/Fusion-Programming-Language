#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>

struct Point {
    int64_t x;
    int64_t y;
};

struct ArcHeader {
    uint64_t ref_count;
};

static struct ArcHeader* arc_header_from_point(struct Point* p) {
    if (!p) {
        return NULL;
    }
    return ((struct ArcHeader*)p) - 1;
}

struct Point* arc_alloc_point(int64_t x, int64_t y) {
    struct ArcHeader* header = (struct ArcHeader*)malloc(sizeof(struct ArcHeader) + sizeof(struct Point));
    if (!header) {
        return NULL;
    }
    header->ref_count = 1;
    struct Point* p = (struct Point*)(header + 1);
    p->x = x;
    p->y = y;
    return p;
}

void arc_retain_point(struct Point* p) {
    struct ArcHeader* header = arc_header_from_point(p);
    if (!header) {
        return;
    }
    header->ref_count += 1;
}

void arc_release_point(struct Point* p) {
    struct ArcHeader* header = arc_header_from_point(p);
    if (!header) {
        return;
    }
    if (header->ref_count > 0) {
        header->ref_count -= 1;
    }
    if (header->ref_count == 0) {
        free(header);
    }
}

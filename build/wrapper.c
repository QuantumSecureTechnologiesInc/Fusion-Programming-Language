#include <stdio.h>

extern int main(void);

void mainCRTStartup(void) {
    printf("[WRAPPER] calling main...\n");
    fflush(stdout);
    int r = main();
    printf("[WRAPPER] main returned: %d\n", r);
}
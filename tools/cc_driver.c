#include "../src/compiler/compiler.h"

#include <stdio.h>

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: %s <src.c> <out.com>\n", argv[0]);
        return 1;
    }
    if (cc_compile(argv[1], argv[2]) != 0) {
        fprintf(stderr, "cc_compile failed for %s -> %s\n", argv[1], argv[2]);
        return 1;
    }
    return 0;
}

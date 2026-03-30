#include "compiler.h"

#include <stdio.h>

int cc_compile(const char *src_path, const char *out_path) {
    FILE *src;
    FILE *out;
    int ch;
    unsigned int checksum = 0u;
    unsigned char blob[8];

    if (src_path == NULL || out_path == NULL) {
        return -1;
    }

    src = fopen(src_path, "rb");
    if (src == NULL) {
        return -1;
    }
    while ((ch = fgetc(src)) != EOF) {
        checksum = (checksum + (unsigned int)(unsigned char)ch) & 0xFFFFu;
    }
    if (fclose(src) != 0) {
        return -1;
    }

    out = fopen(out_path, "wb");
    if (out == NULL) {
        return -1;
    }
    /* Minimal COM stub: NOP; MVI A,checksum_lo; MVI B,checksum_hi; HLT */
    blob[0] = 0x00u;
    blob[1] = 0x3Eu;
    blob[2] = (unsigned char)(checksum & 0xFFu);
    blob[3] = 0x06u;
    blob[4] = (unsigned char)((checksum >> 8) & 0xFFu);
    blob[5] = 0x76u;
    if (fwrite(blob, 1u, 6u, out) != 6u) {
        fclose(out);
        return -1;
    }
    if (fclose(out) != 0) {
        return -1;
    }
    return 0;
}

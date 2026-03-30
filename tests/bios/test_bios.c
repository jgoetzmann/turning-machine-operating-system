#include "../../src/bios/bios.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void assert_u8(const char *label, unsigned int expected, unsigned int actual) {
    if (expected != actual) {
        fprintf(stderr, "FAIL: %s expected=0x%02X actual=0x%02X\n", label, expected, actual);
        exit(1);
    }
}

int main(void) {
    cpu_t cpu = {0};

    bios_init();
    if (bios_pending_output()) {
        fprintf(stderr, "FAIL: queue should be empty after bios_init\n");
        return 1;
    }

    cpu.a = 0x02u; /* CONOUT */
    cpu.c = 'A';
    cpu.io_out_pending = 1u;
    bios_dispatch(&cpu);
    if (!bios_pending_output()) {
        fprintf(stderr, "FAIL: CONOUT should queue one character\n");
        return 1;
    }
    assert_u8("CONOUT drain char", 'A', (unsigned char)bios_get_output());
    if (bios_pending_output()) {
        fprintf(stderr, "FAIL: queue should be empty after drain\n");
        return 1;
    }
    assert_u8("dispatch clears io_out_pending", 0u, cpu.io_out_pending);

    /* Repeated enqueue/dequeue validates ring-index wrap behavior. */
    for (unsigned int i = 0; i < 3000u; ++i) {
        unsigned char expected = (unsigned char)('a' + (i % 26u));
        cpu.a = 0x02u;
        cpu.c = (uint8_t)expected;
        bios_dispatch(&cpu);
        assert_u8("queued character", expected, (unsigned char)bios_get_output());
    }

    /* CONIN should read one char into A. */
    if (ungetc('Q', stdin) == EOF) {
        fprintf(stderr, "FAIL: ungetc setup for CONIN failed\n");
        return 1;
    }
    cpu.a = 0x01u; /* CONIN */
    bios_dispatch(&cpu);
    assert_u8("CONIN stores char in A", 'Q', cpu.a);

    puts("PASS: test_bios");
    return 0;
}

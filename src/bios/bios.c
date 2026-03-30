#include "bios.h"

#include <stdint.h>
#include <stdio.h>

#define BIOS_OUT_CAPACITY 1024

static char g_out[BIOS_OUT_CAPACITY];
static unsigned int g_head = 0;
static unsigned int g_tail = 0;

static unsigned int next_index(unsigned int idx) {
    return (idx + 1u) % BIOS_OUT_CAPACITY;
}

static int out_queue_push(char ch) {
    unsigned int next = next_index(g_head);
    if (next == g_tail) {
        return 0;
    }
    g_out[g_head] = ch;
    g_head = next;
    return 1;
}

static void bios_conin(cpu_t *cpu) {
    int ch = getchar();
    cpu->a = (ch == EOF) ? 0u : (uint8_t)ch;
}

static void bios_conout(cpu_t *cpu) {
    (void)out_queue_push((char)cpu->c);
}

void bios_init(void) {
    g_head = 0;
    g_tail = 0;
}

void bios_dispatch(cpu_t *cpu) {
    if (cpu == NULL) {
        return;
    }

    switch (cpu->a) {
        case 0x01u: /* CONIN */
            bios_conin(cpu);
            break;
        case 0x02u: /* CONOUT */
            bios_conout(cpu);
            break;
        case 0x03u: /* AUXOUT (stub) */
        case 0x04u: /* AUXIN (stub) */
        case 0x09u: /* SELDISK (stub) */
        case 0x0Au: /* SETTRK (stub) */
        case 0x0Bu: /* SETSEC (stub) */
        case 0x0Cu: /* SETDMA (stub) */
        case 0x0Du: /* READ (stub) */
        case 0x0Eu: /* WRITE (stub) */
        default:
            break;
    }

    cpu->io_out_pending = 0u;
}

int bios_pending_output(void) {
    return g_head != g_tail;
}

char bios_get_output(void) {
    char ch = '\0';
    if (g_head != g_tail) {
        ch = g_out[g_tail];
        g_tail = (g_tail + 1u) % BIOS_OUT_CAPACITY;
    }
    return ch;
}

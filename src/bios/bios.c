#include "bios.h"

#include "../emu/mem.h"
#include "../fs/fs.h"

#include <stdint.h>
#include <stdio.h>

#define BIOS_OUT_CAPACITY 1024
#define BIOS_SECTOR_SIZE 256u

static char g_out[BIOS_OUT_CAPACITY];
static unsigned int g_head = 0;
static unsigned int g_tail = 0;
static uint8_t g_disk = 0u;
static uint8_t g_track = 0u;
static uint8_t g_sector = 1u;
static uint16_t g_dma = 0x0080u;

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

static void bios_seldisk(cpu_t *cpu) {
    g_disk = cpu->c;
}

static void bios_settrk(cpu_t *cpu) {
    g_track = cpu->c;
}

static void bios_setsec(cpu_t *cpu) {
    g_sector = cpu->c;
}

static void bios_setdma(cpu_t *cpu) {
    g_dma = (uint16_t)(((uint16_t)cpu->d << 8) | cpu->e);
}

static void bios_read(cpu_t *cpu) {
    uint8_t *mem = mem_raw();
    int rc;
    if ((uint32_t)g_dma + BIOS_SECTOR_SIZE > 65536u) {
        cpu->a = 1u;
        return;
    }
    rc = fs_read_sector(g_track, g_sector, &mem[g_dma]);
    cpu->a = (rc == 0) ? 0u : 1u;
}

static void bios_write(cpu_t *cpu) {
    uint8_t *mem = mem_raw();
    int rc;
    if ((uint32_t)g_dma + BIOS_SECTOR_SIZE > 65536u) {
        cpu->a = 1u;
        return;
    }
    rc = fs_write_sector(g_track, g_sector, &mem[g_dma]);
    cpu->a = (rc == 0) ? 0u : 1u;
}

void bios_init(void) {
    g_head = 0;
    g_tail = 0;
    g_disk = 0u;
    g_track = 0u;
    g_sector = 1u;
    g_dma = 0x0080u;
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
            break;
        case 0x09u: /* SELDISK */
            bios_seldisk(cpu);
            break;
        case 0x0Au: /* SETTRK */
            bios_settrk(cpu);
            break;
        case 0x0Bu: /* SETSEC */
            bios_setsec(cpu);
            break;
        case 0x0Cu: /* SETDMA */
            bios_setdma(cpu);
            break;
        case 0x0Du: /* READ */
            bios_read(cpu);
            break;
        case 0x0Eu: /* WRITE */
            bios_write(cpu);
            break;
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

uint8_t bios_current_disk(void) {
    return g_disk;
}

uint8_t bios_current_track(void) {
    return g_track;
}

uint8_t bios_current_sector(void) {
    return g_sector;
}

uint16_t bios_dma_addr(void) {
    return g_dma;
}

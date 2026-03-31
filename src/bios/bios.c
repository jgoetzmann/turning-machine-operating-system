#include "bios.h"

#include "../compiler/compiler.h"
#include "../emu/mem.h"
#include "../fs/fs.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BIOS_OUT_CAPACITY 1024
#define BIOS_SECTOR_SIZE 256u

static char g_out[BIOS_OUT_CAPACITY];
static unsigned int g_head = 0;
static unsigned int g_tail = 0;
static uint8_t g_disk = 0u;
static uint8_t g_track = 0u;
static uint8_t g_sector = 1u;
static uint16_t g_dma = 0x0080u;

static char g_type_name[256];
static unsigned int g_type_len = 0u;
static int g_run_loaded = 0;

static char g_shell_line[128];
static unsigned int g_shell_line_len = 0u;

#define BIOS_TPA_LOAD 0x0100u
#define BIOS_TPA_END 0x3FFFu

#define CC_STAGE_SRC "build/turingos_cc_src.c"
#define CC_STAGE_OUT "build/turingos_cc_out.com"

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

static void bios_readline(cpu_t *cpu) {
    int ch;
    unsigned int len = 0u;

    g_shell_line_len = 0u;
    g_shell_line[0] = '\0';
    while (1) {
        ch = getchar();
        if (ch == EOF) {
            /* Closed stdin (e.g. turingos </dev/null>): stop the shell loop. */
            cpu->halted = 1;
            return;
        }
        if (ch == 8 || ch == 127) {
            if (len > 0u) {
                len--;
                (void)out_queue_push((char)8);
                (void)out_queue_push(' ');
                (void)out_queue_push((char)8);
            }
            continue;
        }
        if (ch == 10 || ch == 13) {
            break;
        }
        if (len < 128u) {
            g_shell_line[len] = (char)ch;
            len++;
        }
    }
    g_shell_line_len = len;
    if (len < 128u) {
        g_shell_line[len] = '\0';
    } else {
        g_shell_line[127] = '\0';
    }
}

static void bios_lineget(cpu_t *cpu) {
    unsigned int idx = (unsigned int)cpu->c;
    if (idx >= g_shell_line_len || idx >= 128u) {
        cpu->a = 0u;
    } else {
        cpu->a = (uint8_t)(unsigned char)g_shell_line[idx];
    }
}

static void bios_linelen(cpu_t *cpu) {
    cpu->a = (g_shell_line_len > 255u) ? 255u : (uint8_t)g_shell_line_len;
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

static void bios_namech(cpu_t *cpu) {
    if (g_type_len < sizeof(g_type_name) - 1u) {
        g_type_name[g_type_len++] = (char)cpu->c;
    }
}

static void bios_runfile(cpu_t *cpu) {
    uint8_t buf[256];
    int fh;
    int r;
    unsigned int i;
    addr_t addr;

    g_type_name[g_type_len] = '\0';
    g_type_len = 0u;
    if (g_type_name[0] == '\0') {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    fh = fs_open(g_type_name);
    if (fh < 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    addr = BIOS_TPA_LOAD;
    for (;;) {
        r = fs_read(fh, buf, (int)sizeof(buf));
        if (r <= 0) {
            break;
        }
        for (i = 0u; i < (unsigned int)r; i++) {
            if (addr > BIOS_TPA_END) {
                fs_close(fh);
                (void)out_queue_push('?');
                (void)out_queue_push('\n');
                return;
            }
            mem_write(addr, buf[i]);
            addr++;
        }
    }
    fs_close(fh);
    cpu->pc = BIOS_TPA_LOAD;
    cpu->halted = 0;
    g_run_loaded = 1;
}

static void bios_deletefile(cpu_t *cpu) {
    (void)cpu;
    g_type_name[g_type_len] = '\0';
    g_type_len = 0u;
    if (g_type_name[0] == '\0') {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fs_delete(g_type_name) != 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
    }
}

static int bios_derive_com_name(const char *src, char *out, size_t out_cap) {
    size_t n = strlen(src);
    if (n < 3u || n + 6u > out_cap) {
        return -1;
    }
    if (src[n - 2u] != '.' || (src[n - 1u] != 'c' && src[n - 1u] != 'C')) {
        return -1;
    }
    (void)memcpy(out, src, n - 2u);
    out[n - 2u] = '\0';
    if (strlen(out) + 5u > out_cap) {
        return -1;
    }
    (void)strcat(out, ".com");
    return 0;
}

static void bios_cccompile(cpu_t *cpu) {
    char src_name[256];
    char out_name[256];
    uint8_t src_buf[16384];
    uint8_t out_buf[16384];
    int fh;
    int nread;
    int chunk;
    FILE *wfp;
    FILE *rfp;
    long out_sz;
    int wh;
    int w;

    (void)cpu;
    g_type_name[g_type_len] = '\0';
    g_type_len = 0u;
    if (g_type_name[0] == '\0') {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (strlen(g_type_name) >= sizeof(src_name)) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    (void)strcpy(src_name, g_type_name);
    if (bios_derive_com_name(src_name, out_name, sizeof(out_name)) != 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    fh = fs_open(src_name);
    if (fh < 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    nread = 0;
    for (;;) {
        chunk = fs_read(fh, src_buf + nread, (int)sizeof(src_buf) - 1 - nread);
        if (chunk < 0) {
            fs_close(fh);
            (void)out_queue_push('?');
            (void)out_queue_push('\n');
            return;
        }
        if (chunk == 0) {
            break;
        }
        nread += chunk;
        if (nread >= (int)sizeof(src_buf) - 1) {
            fs_close(fh);
            (void)out_queue_push('?');
            (void)out_queue_push('\n');
            return;
        }
    }
    fs_close(fh);
    src_buf[nread] = 0u;

    (void)remove(CC_STAGE_SRC);
    (void)remove(CC_STAGE_OUT);
    wfp = fopen(CC_STAGE_SRC, "wb");
    if (wfp == NULL) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fwrite(src_buf, 1u, (size_t)nread, wfp) != (size_t)nread) {
        (void)fclose(wfp);
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fclose(wfp) != 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }

    if (cc_compile(CC_STAGE_SRC, CC_STAGE_OUT) != 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }

    rfp = fopen(CC_STAGE_OUT, "rb");
    if (rfp == NULL) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fseek(rfp, 0L, SEEK_END) != 0) {
        (void)fclose(rfp);
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    out_sz = ftell(rfp);
    if (out_sz < 0L || out_sz > (long)sizeof(out_buf)) {
        (void)fclose(rfp);
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fseek(rfp, 0L, SEEK_SET) != 0) {
        (void)fclose(rfp);
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fread(out_buf, 1u, (size_t)out_sz, rfp) != (size_t)out_sz) {
        (void)fclose(rfp);
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    if (fclose(rfp) != 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }

    if (fs_exists(out_name)) {
        if (fs_delete(out_name) != 0) {
            (void)out_queue_push('?');
            (void)out_queue_push('\n');
            return;
        }
    }
    wh = fs_create(out_name);
    if (wh < 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    w = fs_write(wh, out_buf, (int)out_sz);
    fs_close(wh);
    if (w != (int)out_sz) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    fs_flush();
}

static void bios_typefile(cpu_t *cpu) {
    uint8_t buf[256];
    int fh;
    int r;
    unsigned int i;

    (void)cpu;
    g_type_name[g_type_len] = '\0';
    g_type_len = 0u;
    if (g_type_name[0] == '\0') {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    fh = fs_open(g_type_name);
    if (fh < 0) {
        (void)out_queue_push('?');
        (void)out_queue_push('\n');
        return;
    }
    for (;;) {
        r = fs_read(fh, buf, (int)sizeof(buf));
        if (r <= 0) {
            break;
        }
        for (i = 0u; i < (unsigned int)r; i++) {
            (void)out_queue_push((char)buf[i]);
        }
    }
    fs_close(fh);
    (void)out_queue_push('\n');
}

void bios_init(void) {
    g_head = 0;
    g_tail = 0;
    g_disk = 0u;
    g_track = 0u;
    g_sector = 1u;
    g_dma = 0x0080u;
    g_type_len = 0u;
    g_type_name[0] = '\0';
    g_run_loaded = 0;
    g_shell_line_len = 0u;
    g_shell_line[0] = '\0';
}

void bios_dispatch(cpu_t *cpu) {
    if (cpu == NULL) {
        return;
    }

    g_run_loaded = 0;

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
        case 0x12u: /* TYPE name byte — append to filename buffer */
            bios_namech(cpu);
            break;
        case 0x13u: /* TYPE end — open file, print contents, clear buffer */
            bios_typefile(cpu);
            break;
        case 0x14u: /* RUN end — load .com into TPA, transfer control */
            bios_runfile(cpu);
            break;
        case 0x15u: /* DEL end — delete file by buffered name */
            bios_deletefile(cpu);
            break;
        case 0x16u: /* CC end — host compile .c from disk to .com on disk */
            bios_cccompile(cpu);
            break;
        case 0x17u: /* READLINE — fill g_shell_line from stdin (backspace ok) */
            bios_readline(cpu);
            break;
        case 0x18u: /* LINEGET — byte g_shell_line[C] into A */
            bios_lineget(cpu);
            break;
        case 0x19u: /* LINELEN — line length into A */
            bios_linelen(cpu);
            break;
        case 0x0Fu: { /* LISTDIR — print directory to console (host fs_list) */
            char names[64][13];
            int n;
            unsigned int i;
            int j;
            n = fs_list(names, 64);
            if (n < 0) {
                (void)out_queue_push('?');
                (void)out_queue_push('\n');
                break;
            }
            if (n == 0) {
                const char *msg = "(empty)\n";
                while (*msg != '\0') {
                    (void)out_queue_push(*msg);
                    msg++;
                }
                break;
            }
            for (i = 0u; (int)i < n; i++) {
                for (j = 0; names[i][j] != '\0' && j < 13; j++) {
                    (void)out_queue_push(names[i][j]);
                }
                (void)out_queue_push('\n');
            }
            break;
        }
        default:
            break;
    }

    cpu->io_out_pending = 0u;
}

int bios_run_program_pending(void) {
    return g_run_loaded;
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

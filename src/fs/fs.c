#include "fs.h"

#include <ctype.h>
#include <stdio.h>
#include <stddef.h>
#include <string.h>

#define FS_TRACKS 77u
#define FS_SECTORS_PER_TRACK 26u
#define FS_BYTES_PER_SECTOR 256u
#define FS_TOTAL_BYTES (FS_TRACKS * FS_SECTORS_PER_TRACK * FS_BYTES_PER_SECTOR)
#define FS_DIR_ENTRY_SIZE 32u
#define FS_DIR_ENTRIES 64u
#define FS_DIR_BYTES (FS_DIR_ENTRY_SIZE * FS_DIR_ENTRIES)
#define FS_OPEN_MAX 16u

static FILE *g_disk_fp = NULL;
static uint8_t g_dir_raw[FS_DIR_BYTES];
static uint8_t g_dir_active[FS_DIR_ENTRIES];
static uint8_t g_open_dir_index[FS_OPEN_MAX];
static uint8_t g_open_in_use[FS_OPEN_MAX];

static long fs_sector_offset(uint8_t track, uint8_t sector) {
    unsigned long linear_sector = ((unsigned long)track * FS_SECTORS_PER_TRACK) +
                                  ((unsigned long)sector - 1u);
    return (long)(linear_sector * FS_BYTES_PER_SECTOR);
}

static void fs_reset_open_table(void) {
    unsigned int i;
    for (i = 0; i < FS_OPEN_MAX; ++i) {
        g_open_in_use[i] = 0u;
        g_open_dir_index[i] = 0u;
    }
}

static int fs_build_cpm_name(const char *name, uint8_t out_name[11]) {
    unsigned int i = 0u;
    unsigned int j = 0u;

    if (name == NULL || name[0] == '\0') {
        return -1;
    }

    for (i = 0u; i < 11u; ++i) {
        out_name[i] = ' ';
    }

    i = 0u;
    while (name[i] != '\0' && name[i] != '.') {
        unsigned char ch = (unsigned char)name[i];
        if (j >= 8u || !isalnum(ch)) {
            return -1;
        }
        out_name[j++] = (uint8_t)toupper(ch);
        i++;
    }
    if (j == 0u) {
        return -1;
    }
    if (name[i] == '.') {
        i++;
        j = 0u;
        while (name[i] != '\0') {
            unsigned char ch = (unsigned char)name[i];
            if (j >= 3u || !isalnum(ch)) {
                return -1;
            }
            out_name[8u + j] = (uint8_t)toupper(ch);
            j++;
            i++;
        }
        if (j == 0u) {
            return -1;
        }
    }
    return 0;
}

static int fs_refresh_directory(void) {
    unsigned int i;
    if (g_disk_fp == NULL) {
        return -1;
    }
    if (fseek(g_disk_fp, 0L, SEEK_SET) != 0) {
        return -1;
    }
    if (fread(g_dir_raw, 1u, FS_DIR_BYTES, g_disk_fp) != FS_DIR_BYTES) {
        return -1;
    }
    for (i = 0u; i < FS_DIR_ENTRIES; ++i) {
        uint8_t status = g_dir_raw[i * FS_DIR_ENTRY_SIZE];
        g_dir_active[i] = (status == 0x00u) ? 1u : 0u;
    }
    return 0;
}

int fs_init(const char *disk_image_path) {
    FILE *fp;
    long size;

    if (disk_image_path == NULL) {
        return -1;
    }

    fp = fopen(disk_image_path, "r+b");
    if (fp == NULL) {
        return -1;
    }

    if (fseek(fp, 0L, SEEK_END) != 0) {
        (void)fclose(fp);
        return -1;
    }
    size = ftell(fp);
    if (size != (long)FS_TOTAL_BYTES) {
        (void)fclose(fp);
        return -1;
    }
    if (fseek(fp, 0L, SEEK_SET) != 0) {
        (void)fclose(fp);
        return -1;
    }

    if (g_disk_fp != NULL) {
        (void)fclose(g_disk_fp);
    }
    g_disk_fp = fp;
    fs_reset_open_table();
    if (fs_refresh_directory() != 0) {
        (void)fclose(g_disk_fp);
        g_disk_fp = NULL;
        return -1;
    }
    return 0;
}

int fs_open(const char *name) {
    uint8_t wanted[11];
    unsigned int i;
    unsigned int h;

    if (g_disk_fp == NULL) {
        return -1;
    }
    if (fs_build_cpm_name(name, wanted) != 0) {
        return -1;
    }
    if (fs_refresh_directory() != 0) {
        return -1;
    }

    for (i = 0u; i < FS_DIR_ENTRIES; ++i) {
        const uint8_t *entry = &g_dir_raw[i * FS_DIR_ENTRY_SIZE];
        if (g_dir_active[i] == 0u) {
            continue;
        }
        if (memcmp(&entry[1], wanted, 11u) != 0) {
            continue;
        }
        for (h = 0u; h < FS_OPEN_MAX; ++h) {
            if (g_open_in_use[h] == 0u) {
                g_open_in_use[h] = 1u;
                g_open_dir_index[h] = (uint8_t)i;
                return (int)h;
            }
        }
        return -1;
    }
    return -1;
}

int fs_read(int fh, uint8_t *buf, int len) {
    (void)fh;
    (void)buf;
    (void)len;
    return -1;
}

int fs_write(int fh, const uint8_t *buf, int len) {
    (void)fh;
    (void)buf;
    (void)len;
    return -1;
}

void fs_close(int fh) {
    if (fh < 0 || fh >= (int)FS_OPEN_MAX) {
        return;
    }
    g_open_in_use[(unsigned int)fh] = 0u;
    g_open_dir_index[(unsigned int)fh] = 0u;
}

int fs_list(char names[][13], int max) {
    (void)names;
    (void)max;
    return 0;
}

int fs_delete(const char *name) {
    (void)name;
    return -1;
}

int fs_exists(const char *name) {
    (void)name;
    return 0;
}

void fs_flush(void) {
    if (g_disk_fp != NULL) {
        (void)fflush(g_disk_fp);
    }
}

int fs_read_sector(uint8_t track, uint8_t sector, uint8_t *buf) {
    long offset;
    if (g_disk_fp == NULL || buf == NULL) {
        return -1;
    }
    if (track >= FS_TRACKS || sector == 0u || sector > FS_SECTORS_PER_TRACK) {
        return -1;
    }

    offset = fs_sector_offset(track, sector);
    if (fseek(g_disk_fp, offset, SEEK_SET) != 0) {
        return -1;
    }
    if (fread(buf, 1u, FS_BYTES_PER_SECTOR, g_disk_fp) != FS_BYTES_PER_SECTOR) {
        return -1;
    }
    return 0;
}

int fs_write_sector(uint8_t track, uint8_t sector, const uint8_t *buf) {
    long offset;
    if (g_disk_fp == NULL || buf == NULL) {
        return -1;
    }
    if (track >= FS_TRACKS || sector == 0u || sector > FS_SECTORS_PER_TRACK) {
        return -1;
    }

    offset = fs_sector_offset(track, sector);
    if (fseek(g_disk_fp, offset, SEEK_SET) != 0) {
        return -1;
    }
    if (fwrite(buf, 1u, FS_BYTES_PER_SECTOR, g_disk_fp) != FS_BYTES_PER_SECTOR) {
        return -1;
    }
    if (fflush(g_disk_fp) != 0) {
        return -1;
    }
    return 0;
}

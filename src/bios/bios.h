#ifndef TURINGOS_BIOS_H
#define TURINGOS_BIOS_H

#include "../emu/cpu.h"

void bios_init(void);
void bios_dispatch(cpu_t *cpu);
int bios_run_program_pending(void);
int bios_pending_output(void);
char bios_get_output(void);
uint8_t bios_current_disk(void);
uint8_t bios_current_track(void);
uint8_t bios_current_sector(void);
uint16_t bios_dma_addr(void);

#endif

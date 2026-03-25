/* kernel/linker.ld
 * Flat 32-bit linker script for the Turing Machine OS kernel.
 * Owned by: BUILD domain (Agent-01), coordinates with HEAD domain.
 *
 * CRITICAL: This is a FLAT 32-bit address space.
 *           NO higher-half mapping. NO virtual memory.
 *           Physical address = virtual address (paging disabled).
 *           Kernel loads at 1MB physical (0x00100000) — standard Multiboot2 load point.
 */

ENTRY(_start)

/* Tape layout (physical addresses) — must match kernel/src/tape/regions.rs */
KERNEL_LOAD_ADDR  = 0x00100000;   /* 1MB — where GRUB loads us */
KERNEL_STACK_SIZE = 0x4000;       /* 16KB initial kernel stack */
KERNEL_STACK_TOP  = 0x00070000;   /* Stack grows DOWN from here (below 1MB) */

SECTIONS
{
    /* Kernel starts at physical 1MB */
    . = KERNEL_LOAD_ADDR;

    _kernel_start = .;

    /* Multiboot2 header MUST be within first 8KB of the image */
    .multiboot2 :
    {
        KEEP(*(.multiboot2))
    }

    /* Kernel executable code */
    .text :
    {
        *(.text._start)    /* Entry point first */
        *(.text .text.*)
    }

    /* Read-only data (string literals, constants, jump tables) */
    .rodata :
    {
        *(.rodata .rodata.*)
    }

    /* Initialized data */
    .data :
    {
        *(.data .data.*)
    }

    /* Uninitialized data — must be zeroed by boot.asm before Rust runs */
    .bss :
    {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
    }

    _kernel_end = .;

    /* ----------------------------------------------------------------
     * Discard sections that will cause linker warnings or bloat the ELF.
     * Exception handling frames are not needed (panic = "abort").
     * ---------------------------------------------------------------- */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note.*)
    }
}

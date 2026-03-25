// kernel/src/tape/regions.rs
// Physical tape region constants — SINGLE SOURCE OF TRUTH.
// Agent: TAPE (Agent-03)
//
// ALL physical addresses in the kernel must come from this file.
// No other file may hardcode a physical address.
// See INV-03 in MASTER.md.
//
// This IS the Turing Machine tape layout.
// Every region has a name, a purpose, and an owning domain.
//
// Layout overview (see MASTER.md Section 5 for full table):
//   0x00000000 – 0x000FFFFF  Low memory (BIOS, VGA, real-mode IVT)
//   0x00100000 – 0x001FFFFF  Kernel code (HEAD loads here)
//   0x00200000 – 0x002FFFFF  Kernel data + BSS
//   0x00300000 – 0x003FFFFF  Kernel heap (TAPE domain)
//   0x00400000 – 0x005FFFFF  Process tape (PROC domain)
//   0x00600000 – 0x006FFFFF  Process stack (PROC domain)
//   0x00700000 – 0x007FFFFF  IPC shared tape (TRANS/IPC domain)
//   0x00800000 – 0x00BFFFFF  Filesystem cache (STORE domain)
//   0x000B8000 – 0x000BFFFF  VGA text buffer (DISPLAY domain — MMIO)

// ─── Tape boundaries ─────────────────────────────────────────────────────

pub const TAPE_START:            u32 = 0x0000_0000;
pub const TAPE_END:              u32 = 0x0100_0000;  // 16MB total tape

// ─── Hardware-mapped tape regions (MMIO) ──────────────────────────────────
// These are NOT RAM — they're device registers mapped into the address space.
// Access with tape_write/tape_read (volatile). NEVER cache these values.

pub const VGA_BASE:              u32 = 0x000B_8000;  // VGA text buffer (DISPLAY domain)
pub const VGA_END:               u32 = 0x000B_FFFF;
pub const VGA_COLS:              u32 = 80;
pub const VGA_ROWS:              u32 = 25;
pub const VGA_SIZE:              u32 = VGA_COLS * VGA_ROWS * 2;  // 2 bytes per cell

// ─── Boot region (BIOS/real-mode) ─────────────────────────────────────────
// Do not write to these — BIOS may need them for interrupt handling.

pub const REAL_MODE_IVT_BASE:    u32 = 0x0000_0000;
pub const REAL_MODE_IVT_END:     u32 = 0x0000_04FF;
pub const BIOS_DATA_AREA_BASE:   u32 = 0x0000_0400;
pub const BIOS_DATA_AREA_END:    u32 = 0x0000_04FF;

// ─── Stack region ─────────────────────────────────────────────────────────
// Kernel stack grows DOWN from KERNEL_STACK_TOP.
// Boot stack set in boot.asm. PROC domain sets up process stacks separately.

pub const KERNEL_STACK_TOP:      u32 = 0x0007_0000;  // 448KB — safe below kernel load
pub const KERNEL_STACK_BOTTOM:   u32 = 0x0006_C000;  // 16KB stack (TOP - 16384)

// ─── Kernel code region ───────────────────────────────────────────────────
// Kernel ELF is loaded here by GRUB/QEMU -kernel.

pub const KERNEL_CODE_BASE:      u32 = 0x0010_0000;  // 1MB — Multiboot2 standard
pub const KERNEL_CODE_END:       u32 = 0x001F_FFFF;  // Up to 1MB of kernel code

// ─── Kernel data region ───────────────────────────────────────────────────

pub const KERNEL_DATA_BASE:      u32 = 0x0020_0000;
pub const KERNEL_DATA_END:       u32 = 0x002F_FFFF;

// ─── Kernel heap region ───────────────────────────────────────────────────
// TAPE domain manages this. Other domains get memory from here via heap allocator.

pub const KERNEL_HEAP_BASE:      u32 = 0x0030_0000;
pub const KERNEL_HEAP_END:       u32 = 0x003F_FFFF;  // 1MB kernel heap
pub const KERNEL_HEAP_SIZE:      u32 = KERNEL_HEAP_END - KERNEL_HEAP_BASE + 1;

// ─── Process tape region ──────────────────────────────────────────────────
// Process code + data loaded here by PROC domain.
// Phase 1: one process at a time (single slot).

pub const PROC_TAPE_BASE:        u32 = 0x0040_0000;
pub const PROC_TAPE_END:         u32 = 0x005F_FFFF;  // 2MB per process slot
pub const PROC_TAPE_SIZE:        u32 = PROC_TAPE_END - PROC_TAPE_BASE + 1;

pub const PROC_STACK_TOP:        u32 = 0x0070_0000;  // User stack top (grows DOWN)
pub const PROC_STACK_BOTTOM:     u32 = 0x0060_0000;  // 1MB user stack

// ─── IPC / shared tape region ─────────────────────────────────────────────
// Shared memory between kernel and userspace (Phase 2).

pub const IPC_TAPE_BASE:         u32 = 0x0070_0000;
pub const IPC_TAPE_END:          u32 = 0x007F_FFFF;  // 1MB IPC region

// ─── Filesystem cache region ──────────────────────────────────────────────
// STORE domain caches disk sectors here.

pub const FS_CACHE_BASE:         u32 = 0x0080_0000;
pub const FS_CACHE_END:          u32 = 0x00BF_FFFF;  // 4MB cache
pub const FS_CACHE_SIZE:         u32 = FS_CACHE_END - FS_CACHE_BASE + 1;

// ─── Validation helpers ───────────────────────────────────────────────────

/// Check if a physical address is in the VGA tape region (MMIO).
pub fn is_vga_region(addr: u32) -> bool {
    addr >= VGA_BASE && addr <= VGA_END
}

/// Check if a physical address is in usable kernel heap tape.
pub fn is_heap_region(addr: u32) -> bool {
    addr >= KERNEL_HEAP_BASE && addr <= KERNEL_HEAP_END
}

/// Check if a physical address is in the process tape region.
pub fn is_proc_tape(addr: u32) -> bool {
    addr >= PROC_TAPE_BASE && addr <= PROC_TAPE_END
}

/// Validate a range [base, base+len) is within a single tape region.
/// Returns false if the range crosses region boundaries or is out of bounds.
pub fn is_valid_range(base: u32, len: u32) -> bool {
    if len == 0 { return true; }
    let end = match base.checked_add(len) {
        Some(e) => e,
        None => return false,  // overflow
    };

    // Must be within total tape bounds
    if base < TAPE_START || end > TAPE_END { return false; }

    // Must not span across the BIOS gap (0xA0000-0xFFFFF)
    if base < 0x000A_0000 && end > 0x000A_0000 { return false; }

    true
}

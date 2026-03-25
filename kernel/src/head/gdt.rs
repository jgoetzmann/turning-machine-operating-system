// kernel/src/head/gdt.rs
// Global Descriptor Table — flat 32-bit segmentation model.
// Agent: HEAD (Agent-02)
//
// "Flat" means every segment has base=0 and limit=4GB.
// All segments cover the entire 32-bit address space.
// There is no memory protection via segmentation — the tape is flat.
// This is intentional: we are a Turing Machine, the tape is one flat address space.
//
// GDT layout:
//   Index 0 (selector 0x00): Null descriptor    — required by x86
//   Index 1 (selector 0x08): Kernel code (DPL=0) — CS after boot
//   Index 2 (selector 0x10): Kernel data (DPL=0) — DS/SS after boot
//   Index 3 (selector 0x18): User code   (DPL=3) — CS for userspace
//   Index 4 (selector 0x20): User data   (DPL=3) — DS for userspace
//   Index 5 (selector 0x28): TSS                 — for ring 3 → ring 0 stack switch
//
// Selector = (index << 3) | RPL | TI
//   RPL=0 (ring 0), TI=0 (GDT), so: selector = index * 8

pub const KERNEL_CODE_SEL: u16 = 0x08;
pub const KERNEL_DATA_SEL: u16 = 0x10;
pub const USER_CODE_SEL:   u16 = 0x18 | 3;  // DPL=3 in selector RPL bits
pub const USER_DATA_SEL:   u16 = 0x20 | 3;
pub const TSS_SEL:         u16 = 0x28;

/// A single GDT entry (descriptor), packed as a u64.
/// Bit layout is complex — see Intel SDM Vol 3A Section 3.4.5.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low:  u16,   // bits 15:0 of limit
    base_low:   u16,   // bits 15:0 of base
    base_mid:   u8,    // bits 23:16 of base
    access:     u8,    // access byte (type, DPL, present)
    gran:       u8,    // granularity byte (limit_high, flags)
    base_high:  u8,    // bits 31:24 of base
}

impl GdtEntry {
    /// Null descriptor — index 0 must always be null.
    pub const fn null() -> Self {
        Self { limit_low: 0, base_low: 0, base_mid: 0, access: 0, gran: 0, base_high: 0 }
    }

    /// Create a flat 32-bit segment descriptor.
    ///
    /// `dpl`: 0 = kernel, 3 = user
    /// `executable`: true = code segment, false = data segment
    pub const fn flat_32(dpl: u8, executable: bool) -> Self {
        // Access byte:
        //   bit 7: Present (must be 1)
        //   bits 6:5: DPL (0=ring0, 3=ring3)
        //   bit 4: Descriptor type (1 = code/data, 0 = system)
        //   bit 3: Executable (1 = code, 0 = data)
        //   bit 2: Direction/Conforming (0 = normal)
        //   bit 1: Read/Write (1 = readable code / writable data)
        //   bit 0: Accessed (CPU sets this — start as 0)
        let access: u8 = 0b1000_0010                  // present + type + writable
            | ((dpl & 0x3) << 5)                      // DPL
            | if executable { 0b0000_1000 } else { 0 }; // executable bit

        // Granularity byte:
        //   bit 7: Granularity (1 = limit is in 4KB pages)
        //   bit 6: D/B (1 = 32-bit default operand size)
        //   bit 5: L (0 = not 64-bit) — must be 0 for 32-bit
        //   bit 4: AVL (available for OS use)
        //   bits 3:0: limit bits 19:16 (0xF for 4GB)
        let gran: u8 = 0b1100_1111;  // 4KB granularity, 32-bit, limit=0xFFFFF

        Self {
            limit_low: 0xFFFF,   // limit bits 15:0 = 0xFFFF
            base_low:  0x0000,   // base = 0
            base_mid:  0x00,
            access,
            gran,
            base_high: 0x00,
        }
    }
}

/// GDTR value — pointer to the GDT, loaded with lgdt instruction.
#[repr(C, packed)]
pub struct GdtDescriptor {
    pub limit: u16,  // size of GDT in bytes - 1
    pub base:  u32,  // physical address of GDT
}

/// The Global Descriptor Table.
/// Static so it lives in .data and has a fixed address.
static mut GDT: [GdtEntry; 6] = [
    GdtEntry::null(),              // 0x00 — null (required)
    GdtEntry::flat_32(0, true),    // 0x08 — kernel code (DPL=0, executable)
    GdtEntry::flat_32(0, false),   // 0x10 — kernel data (DPL=0, writable)
    GdtEntry::flat_32(3, true),    // 0x18 — user code   (DPL=3, executable)
    GdtEntry::flat_32(3, false),   // 0x20 — user data   (DPL=3, writable)
    GdtEntry::null(),              // 0x28 — TSS placeholder (HEAD fills later)
];

static mut GDTR: GdtDescriptor = GdtDescriptor { limit: 0, base: 0 };

/// Initialize the GDT descriptor with the correct base and limit.
/// Must be called before gdt_load().
pub fn gdt_init() {
    // SAFETY: We are in single-threaded boot context. No races possible.
    unsafe {
        GDTR = GdtDescriptor {
            limit: (core::mem::size_of::<[GdtEntry; 6]>() - 1) as u16,
            base:  GDT.as_ptr() as u32,
        };
    }
}

/// Load the GDT and reload all segment registers.
///
/// After lgdt, CS still has the old value — we must do a far jump to flush it.
/// Then reload DS/ES/FS/GS/SS explicitly.
///
/// See MISTAKE-006 in docs/MISTAKES.md — not doing this causes #GP faults.
pub fn gdt_load() {
    unsafe {
        core::arch::asm!(
            // Load the new GDT
            "lgdt [{gdtr}]",

            // Far jump to reload CS with kernel code selector (0x08)
            // This is the only way to reload the CS register.
            "ljmp $0x08, $2f",
            "2:",

            // Reload all data segment registers with kernel data selector (0x10)
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",

            gdtr = in(reg) &GDTR as *const GdtDescriptor as u32,
            // We can't use named registers for the far jump target,
            // so we use the hard-coded selector 0x08 above.
            options(nostack)
        );
    }
}

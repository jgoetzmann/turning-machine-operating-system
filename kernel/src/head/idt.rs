// kernel/src/head/idt.rs
// Interrupt Descriptor Table — 32-bit protected mode IDT.
// Agent: HEAD (Agent-02) owns the IDT structure.
// Agent: TRANS (Agent-05) fills in the actual gate handlers.
//
// HEAD initializes an empty IDT (all gates = null/not-present).
// TRANS calls idt_set_gate() for each handler during machine_boot().
//
// 32-bit IDT gate descriptor format (8 bytes each):
//   bits 15:0   — handler offset low
//   bits 31:16  — segment selector (must be KERNEL_CODE_SEL = 0x08)
//   bits 39:32  — reserved (must be 0)
//   bits 47:40  — gate type + DPL + present bit
//   bits 63:48  — handler offset high
//
// Gate types:
//   0x8E = interrupt gate, DPL=0, present (clears IF on entry)
//   0x8F = trap gate,      DPL=0, present (does NOT clear IF)
//   0xEE = interrupt gate, DPL=3, present (user-accessible — for int 0x80)

pub const IDT_SIZE: usize = 256;

/// A single 32-bit IDT gate descriptor.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtGate {
    offset_low:  u16,   // handler offset bits 15:0
    selector:    u16,   // code segment selector (0x08 = kernel code)
    _reserved:   u8,    // always 0
    type_attr:   u8,    // gate type + DPL + present
    offset_high: u16,   // handler offset bits 31:16
}

impl IdtGate {
    pub const fn null() -> Self {
        Self { offset_low: 0, selector: 0, _reserved: 0, type_attr: 0, offset_high: 0 }
    }

    /// Create a gate pointing to `handler` in the kernel code segment.
    /// `flags`: 0x8E = interrupt gate DPL=0, 0xEE = interrupt gate DPL=3
    pub fn new(handler: u32, flags: u8) -> Self {
        Self {
            offset_low:  (handler & 0xFFFF) as u16,
            selector:    crate::head::gdt::KERNEL_CODE_SEL,
            _reserved:   0,
            type_attr:   flags,
            offset_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

/// Gate type constants
pub const GATE_INTERRUPT_KERN: u8 = 0x8E;  // Interrupt gate, DPL=0 (IF cleared)
pub const GATE_TRAP_KERN:      u8 = 0x8F;  // Trap gate,      DPL=0 (IF not cleared)
pub const GATE_INTERRUPT_USER: u8 = 0xEE;  // Interrupt gate, DPL=3 (for int 0x80)

/// IDTR — pointer to the IDT, loaded with lidt.
#[repr(C, packed)]
pub struct IdtDescriptor {
    pub limit: u16,
    pub base:  u32,
}

/// The IDT — 256 entries, all null on init.
static mut IDT: [IdtGate; IDT_SIZE] = [IdtGate::null(); IDT_SIZE];
static mut IDTR: IdtDescriptor = IdtDescriptor { limit: 0, base: 0 };

/// Initialize the IDT with all-null entries and load it.
/// TRANS domain will fill gates via idt_set_gate() later.
pub fn idt_init() {
    unsafe {
        IDTR = IdtDescriptor {
            limit: (core::mem::size_of::<[IdtGate; IDT_SIZE]>() - 1) as u16,
            base:  IDT.as_ptr() as u32,
        };
        core::arch::asm!("lidt [{idtr}]", idtr = in(reg) &IDTR as *const IdtDescriptor as u32);
    }
    // NOTE: All entries are null/not-present. Any interrupt at this point
    // causes a double fault (#DF), which causes a triple fault = QEMU reset.
    // TRANS must register handlers before enabling interrupts (sti).
}

/// Set an IDT gate. Called by TRANS domain during machine_boot().
///
/// # Safety
/// `handler` must be a valid kernel function address.
/// `vector` must be 0..255.
pub unsafe fn idt_set_gate(vector: u8, handler: u32, flags: u8) {
    IDT[vector as usize] = IdtGate::new(handler, flags);
}

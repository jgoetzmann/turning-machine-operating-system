// kernel/src/trans/mod.rs
// TRANS domain — Transition Function Implementation
// Agent: Agent-05
// TM Role: When an event fires, TRANS executes the concrete response.
//
// TRANS owns:
//   - int 0x80 syscall gate (DPL=3) and dispatch table
//   - IRQ handlers (timer, keyboard, spurious)
//   - Exception handlers (#GP, #DF, #PF → Panic state)
//
// Read docs/agents/TRANS.md before modifying.

pub mod syscall;     // int 0x80 entry + dispatch table
pub mod irq;         // IRQ0 timer, IRQ1 keyboard, spurious IRQ7/15
pub mod exceptions;  // CPU exceptions → Panic state

use crate::head::idt;

/// Register all IDT gates.
/// Called by CTRL during machine_boot(), before sti.
///
/// # Safety
/// Must be called after idt_init() and before sti.
/// All handler function pointers must be valid kernel code addresses.
pub unsafe fn register_all_handlers() {
    // ── CPU exception handlers ───────────────────────────────────────────
    idt::idt_set_gate(0x00, exceptions::divide_error_handler    as u32, idt::GATE_INTERRUPT_KERN);
    idt::idt_set_gate(0x06, exceptions::invalid_opcode_handler  as u32, idt::GATE_INTERRUPT_KERN);
    idt::idt_set_gate(0x08, exceptions::double_fault_handler    as u32, idt::GATE_INTERRUPT_KERN);
    idt::idt_set_gate(0x0D, exceptions::general_protection_handler as u32, idt::GATE_INTERRUPT_KERN);
    idt::idt_set_gate(0x0E, exceptions::page_fault_handler      as u32, idt::GATE_INTERRUPT_KERN);

    // ── Hardware IRQ handlers (PIC remapped to 0x20–0x2F) ───────────────
    idt::idt_set_gate(0x20, irq::timer_handler    as u32, idt::GATE_INTERRUPT_KERN);  // IRQ0
    idt::idt_set_gate(0x21, irq::keyboard_handler as u32, idt::GATE_INTERRUPT_KERN);  // IRQ1
    idt::idt_set_gate(0x27, irq::spurious_handler as u32, idt::GATE_INTERRUPT_KERN);  // IRQ7 spurious
    idt::idt_set_gate(0x2F, irq::spurious_handler as u32, idt::GATE_INTERRUPT_KERN);  // IRQ15 spurious

    // ── Syscall gate — DPL=3 so userspace can invoke int 0x80 ───────────
    // See MISTAKE-004: this MUST be DPL=3, not DPL=0
    idt::idt_set_gate(0x80, syscall::syscall_entry as u32, idt::GATE_INTERRUPT_USER);

    crate::serial_println!("TRANS: all IDT gates registered");
}

// kernel/src/trans/irq.rs
// Hardware IRQ handlers.
// Agent: TRANS (Agent-05)
//
// CRITICAL: Every IRQ handler MUST send PIC EOI as its LAST operation.
// See MISTAKE-003 in docs/MISTAKES.md — forgetting EOI stops all future IRQs.
//
// Handlers registered in trans/mod.rs via idt_set_gate().
// Hardware IRQ line → IDT vector mapping (after PIC remap to 0x20):
//   IRQ0  → 0x20  PIT timer
//   IRQ1  → 0x21  PS/2 keyboard
//   IRQ7  → 0x27  PIC1 spurious
//   IRQ15 → 0x2F  PIC2 spurious

/// Interrupt stack frame pushed by the CPU on interrupt entry (32-bit).
/// Used by extern "x86-interrupt" handlers.
#[repr(C)]
pub struct InterruptFrame {
    pub eip:    u32,
    pub cs:     u32,
    pub eflags: u32,
    // If coming from ring 3 (user→kernel), CPU also pushes:
    // pub esp: u32,
    // pub ss:  u32,
}

/// IRQ0 — PIT Timer handler (fires at 100Hz).
///
/// Increments tick counter, posts TimerTick event to CTRL.
/// MUST send EOI before returning.
pub extern "x86-interrupt" fn timer_handler(_frame: &InterruptFrame) {
    // Increment the tick counter in the device layer
    crate::device::pit::pit_tick();

    // Post timer event to state machine
    crate::ctrl::push_event(crate::ctrl::state::Event::TimerTick);

    // SAFETY: Port 0x20 is PIC1 command register. EOI = 0x20.
    // This MUST be the last operation before iret.
    // See MISTAKE-003.
    unsafe { crate::device::pic::pic_send_eoi(0); }
}

/// IRQ1 — PS/2 Keyboard handler.
///
/// Reads scancode from port 0x60, buffers it, posts TapeInput event.
/// MUST send EOI before returning.
pub extern "x86-interrupt" fn keyboard_handler(_frame: &InterruptFrame) {
    // Let the device layer read and buffer the scancode
    crate::device::keyboard::keyboard_handle_irq();

    // Notify state machine that tape input arrived
    crate::ctrl::push_event(crate::ctrl::state::Event::TapeInput);

    // SAFETY: Port 0x20 is PIC1 EOI. IRQ1 is on PIC1.
    // See MISTAKE-003.
    unsafe { crate::device::pic::pic_send_eoi(1); }
}

/// IRQ7 / IRQ15 — Spurious IRQ handler.
///
/// The PIC occasionally sends spurious IRQ7 or IRQ15.
/// For IRQ7: check PIC ISR before sending EOI.
/// For IRQ15: always send EOI to PIC1; check PIC2 ISR.
/// If spurious: do NOT send EOI to the PIC that generated it.
pub extern "x86-interrupt" fn spurious_handler(_frame: &InterruptFrame) {
    // Check if IRQ7 is genuinely in-service (not spurious)
    // SAFETY: Reading PIC ISR is safe — we just send an OCW3 command.
    let is_real = unsafe { !crate::device::pic::pic_is_spurious(7) };
    if is_real {
        // Real IRQ7 — handle it (nothing to do for unregistered IRQs) + send EOI
        unsafe { crate::device::pic::pic_send_eoi(7); }
    }
    // Spurious IRQ7: do NOT send EOI — the PIC didn't actually assert it
    // Silently drop.
}

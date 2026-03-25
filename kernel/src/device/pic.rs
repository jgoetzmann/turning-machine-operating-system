// kernel/src/device/pic.rs
// 8259 Programmable Interrupt Controller
// Agent: DEVICE (Agent-06)
//
// Port reference:
//   PIC1 command: 0x20  |  PIC1 data: 0x21
//   PIC2 command: 0xA0  |  PIC2 data: 0xA1

const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const PIC_EOI:   u8  = 0x20;  // End-of-interrupt command

/// x86 port I/O helpers
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
                     options(nomem, nostack, preserves_flags));
}
unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port,
                     options(nomem, nostack, preserves_flags));
    v
}
/// Short I/O delay (write to port 0x80 — POST diagnostic port, safe to write)
unsafe fn io_wait() {
    outb(0x80, 0);
}

/// Initialize and remap the 8259 PIC.
///
/// Default IRQ vectors 0x00–0x0F conflict with CPU exception vectors.
/// This remaps:
///   PIC1: IRQ 0-7  → vectors offset1..offset1+7  (typically 0x20-0x27)
///   PIC2: IRQ 8-15 → vectors offset2..offset2+7  (typically 0x28-0x2F)
///
/// # Safety
/// Must be called before `sti`. Call only once.
pub unsafe fn pic_init(offset1: u8, offset2: u8) {
    // Save existing interrupt masks
    let mask1 = inb(PIC1_DATA);
    let mask2 = inb(PIC2_DATA);

    // ICW1: start initialization sequence, cascade mode, ICW4 needed
    outb(PIC1_CMD,  0x11); io_wait();
    outb(PIC2_CMD,  0x11); io_wait();

    // ICW2: vector offsets
    outb(PIC1_DATA, offset1); io_wait();
    outb(PIC2_DATA, offset2); io_wait();

    // ICW3: cascade identity
    outb(PIC1_DATA, 0x04); io_wait();  // PIC1: IR2 connects to PIC2
    outb(PIC2_DATA, 0x02); io_wait();  // PIC2: cascade identity = 2

    // ICW4: 8086 mode
    outb(PIC1_DATA, 0x01); io_wait();
    outb(PIC2_DATA, 0x01); io_wait();

    // Restore saved masks
    outb(PIC1_DATA, mask1);
    outb(PIC2_DATA, mask2);
}

/// Send End-of-Interrupt signal for an IRQ.
/// Must be called at the END of every IRQ handler.
/// See MISTAKE-003 — forgetting this stops all future IRQs.
///
/// # Safety
/// Must only be called from within an IRQ handler.
pub unsafe fn pic_send_eoi(irq: u8) {
    if irq >= 8 {
        // IRQ 8-15 are on PIC2 — send EOI to PIC2 first, then PIC1
        outb(PIC2_CMD, PIC_EOI);
    }
    outb(PIC1_CMD, PIC_EOI);
}

/// Check if an IRQ is spurious (not actually asserted by hardware).
/// Returns true if the IRQ is spurious and should NOT receive an EOI.
///
/// Only relevant for IRQ7 (PIC1 spurious) and IRQ15 (PIC2 spurious).
///
/// # Safety
/// Safe to call from IRQ handler context.
pub unsafe fn pic_is_spurious(irq: u8) -> bool {
    // Send OCW3 to read In-Service Register (ISR)
    if irq == 7 {
        outb(PIC1_CMD, 0x0B);  // OCW3: read ISR
        let isr = inb(PIC1_CMD);
        (isr & 0x80) == 0  // bit 7 = IRQ7; if 0, it's spurious
    } else if irq == 15 {
        outb(PIC2_CMD, 0x0B);
        let isr = inb(PIC2_CMD);
        (isr & 0x80) == 0
    } else {
        false
    }
}

/// Mask (disable) an IRQ line.
pub unsafe fn pic_mask(irq: u8) {
    let (port, bit) = if irq < 8 { (PIC1_DATA, irq) } else { (PIC2_DATA, irq - 8) };
    let mask = inb(port);
    outb(port, mask | (1 << bit));
}

/// Unmask (enable) an IRQ line.
pub unsafe fn pic_unmask(irq: u8) {
    let (port, bit) = if irq < 8 { (PIC1_DATA, irq) } else { (PIC2_DATA, irq - 8) };
    let mask = inb(port);
    outb(port, mask & !(1 << bit));
}

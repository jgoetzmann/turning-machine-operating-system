// kernel/src/device/mod.rs
// DEVICE domain — Hardware I/O devices
// Agent: Agent-06
// Read docs/agents/DEVICE.md before modifying.

pub mod pic;       // 8259 PIC — interrupt controller
pub mod pit;       // 8253 PIT — timer (100Hz system tick)
pub mod keyboard;  // PS/2 keyboard — scancode → ASCII
pub mod ata;       // ATA PIO — disk read/write

/// Initialize all hardware devices.
/// Called by CTRL during machine_boot(), before sti.
///
/// Order matters:
/// 1. PIC FIRST — remap IRQ vectors before enabling any IRQ
/// 2. PIT — set timer frequency
/// 3. Keyboard — enable keyboard IRQ
/// 4. ATA — detect disk (non-blocking)
pub fn init_all_devices() {
    // Step 1: Remap PIC to 0x20/0x28 BEFORE enabling interrupts
    // Without this, IRQ0-7 collide with CPU exception vectors 0-7
    unsafe { pic::pic_init(0x20, 0x28); }
    crate::serial_println!("DEVICE: PIC remapped (IRQ0-7 → 0x20-0x27)");

    // Step 2: Initialize PIT at 100Hz
    pit::pit_init(100);
    crate::serial_println!("DEVICE: PIT initialized at 100Hz");

    // Step 3: Initialize keyboard
    keyboard::keyboard_init();
    crate::serial_println!("DEVICE: PS/2 keyboard initialized");

    // Step 4: Probe ATA disk (best-effort — no disk is OK for some tests)
    if ata::ata_init() {
        crate::serial_println!("DEVICE: ATA disk detected");
    } else {
        crate::serial_println!("DEVICE: no ATA disk (filesystem tests will fail)");
    }
}

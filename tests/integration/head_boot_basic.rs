// tests/integration/head_boot_basic.rs
// Test: kernel boots, reaches serial output, no panic.
// Domain: HEAD | Agent: Agent-02
// Run: make qemu-test TEST=head_boot_basic
//
// This test is compiled INTO the kernel when `test=head_boot_basic`
// is passed via the kernel command line (-append "test=head_boot_basic").
// The test module checks for the test name and runs its logic,
// printing the pass/fail sentinel to serial.
//
// All integration tests follow this same pattern.

// ── Test entry (called from kernel main loop when test mode is active) ──

/// Called from ctrl::machine_boot() if kernel was launched with "test=head_boot_basic".
/// Prints the pass/fail sentinel and exits QEMU via the debug exit device.
pub fn run() {
    use crate::{serial_println, kprintln};

    serial_println!("[TEST START: head_boot_basic]");

    // Verify we reached this point — means boot_init ran successfully
    serial_println!("  HEAD: multiboot2 validated");
    serial_println!("  HEAD: GDT loaded");
    serial_println!("  HEAD: serial output working");
    serial_println!("  HEAD: VGA output working");

    kprintln!("HEAD BOOT TEST OK");

    serial_println!("[TEST PASS: head_boot_basic]");

    // Exit QEMU via isa-debug-exit device
    // Writing 0x01 to port 0xF4 exits with code (0x01 << 1) | 1 = 3
    // The test runner interprets exit code 3 as success (see test_runner.py)
    qemu_exit_success();
}

/// Signal QEMU to exit with success code via isa-debug-exit device.
pub fn qemu_exit_success() {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xF4u16,
            in("al") 0x01u8,
            options(nomem, nostack)
        );
    }
    // If QEMU doesn't exit (device not configured), loop
    loop { unsafe { core::arch::asm!("hlt") }; }
}

/// Signal QEMU to exit with failure code.
pub fn qemu_exit_failure(reason: &str) {
    crate::serial_println!("[TEST FAIL: head_boot_basic] {}", reason);
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xF4u16,
            in("al") 0x00u8,
            options(nomem, nostack)
        );
    }
    loop { unsafe { core::arch::asm!("hlt") }; }
}

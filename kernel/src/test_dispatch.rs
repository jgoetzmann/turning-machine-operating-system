// kernel/src/test_dispatch.rs
// Test mode dispatcher — routes to the correct integration test.
// Only compiled in when the feature "integration-tests" is enabled,
// or when the kernel detects "test=NAME" in the command line.
//
// The kernel checks for this at boot and calls test_dispatch::run_if_test_mode()
// from ctrl::machine_boot(). If a test name is found, the test runs and
// calls qemu_exit_success/failure — the kernel never enters the main loop.

/// Check the Multiboot2 command line for "test=NAME".
/// If found, run the named test and exit QEMU. Never returns.
/// If not found, return normally (kernel enters main loop).
pub fn run_if_test_mode(cmdline: Option<&str>) {
    let cmdline = match cmdline {
        Some(s) => s,
        None => return,
    };

    let test_name = cmdline
        .split_whitespace()
        .find_map(|token| token.strip_prefix("test="));

    let test_name = match test_name {
        Some(name) => name,
        None => return,
    };

    crate::serial_println!("TEST MODE: running test '{}'", test_name);

    match test_name {
        "head_boot_basic"    => crate::tests::head_boot_basic::run(),
        "tape_alloc"         => crate::tests::tape_alloc::run(),
        "tape_regions"       => crate::tests::device_timer::tape_regions::run(),
        "ctrl_states"        => crate::tests::ctrl_states::run(),
        "display_write"      => crate::tests::display_write::run(),
        "device_timer"       => crate::tests::device_timer::run(),
        "store_fat12"        => crate::tests::store_fat12::run(),
        "proc_create"        => crate::tests::proc_create::run(),
        "shell_echo"         => crate::tests::device_timer::shell_echo::run(),

        unknown => {
            crate::serial_println!("TEST MODE: unknown test '{}'", unknown);
            crate::serial_println!("[TEST FAIL: {}] test not found", unknown);
            crate::tests::head_boot_basic::qemu_exit_failure("test not found");
        }
    }

    // Tests call qemu_exit_success/failure and never return.
    // This loop is unreachable but required for the type checker.
    unreachable!("test should have exited QEMU");
}

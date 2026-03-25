// tests/integration/device_timer.rs
// Test: PIT fires at ~100Hz; tick counter increments.
// Domain: DEVICE | Run: make qemu-test TEST=device_timer

pub fn run() {
    crate::serial_println!("[TEST START: device_timer]");

    let t0 = crate::device::pit::pit_get_ticks();
    crate::serial_println!("  DEVICE: initial ticks = {}", t0);

    // Enable interrupts and wait ~200ms (should see ~20 ticks at 100Hz)
    unsafe { core::arch::asm!("sti") };

    // Busy spin ~200ms worth of iterations (rough, no calibration needed)
    // On QEMU ~500MHz effective speed, 200ms ≈ 100_000_000 iterations
    let mut spins: u32 = 0;
    let target_ticks = t0 + 15;  // wait for at least 15 ticks
    loop {
        spins = spins.wrapping_add(1);
        let t = crate::device::pit::pit_get_ticks();
        if t >= target_ticks { break; }
        if spins > 100_000_000 { break; }  // timeout
        unsafe { core::arch::asm!("pause") };
    }

    let t1 = crate::device::pit::pit_get_ticks();
    let elapsed = t1 - t0;
    crate::serial_println!("  DEVICE: after wait: ticks = {} (elapsed = {})", t1, elapsed);

    if elapsed < 10 {
        crate::serial_println!("[TEST FAIL: device_timer] only {} ticks elapsed (expected ≥10)", elapsed);
        super::head_boot_basic::qemu_exit_failure("too few ticks");
    }

    crate::serial_println!("[TEST PASS: device_timer]");
    super::head_boot_basic::qemu_exit_success();
}

// ─────────────────────────────────────────────────────────────────────────────

// tests/integration/tape_regions.rs
// Test: tape region constants are sane (non-overlapping, within 16MB).
// Domain: TAPE | Run: make qemu-test TEST=tape_regions

pub mod tape_regions {
    pub fn run() {
        use crate::tape::regions::*;
        crate::serial_println!("[TEST START: tape_regions]");

        // All regions must be within TAPE_START..TAPE_END (16MB)
        assert!(KERNEL_CODE_BASE >= TAPE_START);
        assert!(KERNEL_HEAP_END  <  TAPE_END);
        assert!(PROC_TAPE_END    <  TAPE_END);
        assert!(FS_CACHE_END     <  TAPE_END);

        // VGA is below 1MB (MMIO region)
        assert!(VGA_BASE < 0x00100000);

        // Kernel loads at 1MB
        assert_eq!(KERNEL_CODE_BASE, 0x00100000);

        // Heap is above kernel code
        assert!(KERNEL_HEAP_BASE > KERNEL_CODE_BASE);

        // Process tape is above heap
        assert!(PROC_TAPE_BASE > KERNEL_HEAP_END);

        // Stack grows down — stack top > stack bottom
        assert!(KERNEL_STACK_TOP > KERNEL_STACK_BOTTOM);
        assert!(PROC_STACK_TOP > PROC_TAPE_END);

        // Regions don't overlap
        assert!(KERNEL_HEAP_BASE > KERNEL_DATA_END,
            "heap overlaps data: {:#x} > {:#x}", KERNEL_HEAP_BASE, KERNEL_DATA_END);
        assert!(PROC_TAPE_BASE >= KERNEL_HEAP_END,
            "proc tape overlaps heap");
        assert!(FS_CACHE_BASE >= IPC_TAPE_END,
            "fs cache overlaps IPC tape");

        crate::serial_println!("  TAPE: VGA    = {:#010x}", VGA_BASE);
        crate::serial_println!("  TAPE: KERNEL = {:#010x}", KERNEL_CODE_BASE);
        crate::serial_println!("  TAPE: HEAP   = {:#010x}–{:#010x}", KERNEL_HEAP_BASE, KERNEL_HEAP_END);
        crate::serial_println!("  TAPE: PROC   = {:#010x}–{:#010x}", PROC_TAPE_BASE, PROC_TAPE_END);
        crate::serial_println!("  TAPE: FS     = {:#010x}–{:#010x}", FS_CACHE_BASE, FS_CACHE_END);

        crate::serial_println!("[TEST PASS: tape_regions]");
        super::super::head_boot_basic::qemu_exit_success();
    }
}

// ─────────────────────────────────────────────────────────────────────────────

// tests/integration/shell_echo.rs
// Test: shell echo command writes to VGA.
// Domain: SHELL | Run: make qemu-test TEST=shell_echo

pub mod shell_echo {
    pub fn run() {
        crate::serial_println!("[TEST START: shell_echo]");

        // Simulate what the shell does: call sys_write with "hello tape\n"
        let msg = b"hello tape\n";
        let result = crate::trans::syscall::syscall_dispatch(
            crate::trans::syscall::nr::WRITE,
            1,               // fd=1 stdout
            msg.as_ptr() as u32,
            msg.len() as u32,
        );

        if result != msg.len() as u32 {
            crate::serial_println!("[TEST FAIL: shell_echo] sys_write returned {}", result);
            super::super::head_boot_basic::qemu_exit_failure("sys_write failed");
        }

        crate::serial_println!("[TEST PASS: shell_echo]");
        super::super::head_boot_basic::qemu_exit_success();
    }
}

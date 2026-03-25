// tests/integration/tape_alloc.rs
// Test: bump allocator returns valid, non-overlapping addresses.
// Domain: TAPE | Run: make qemu-test TEST=tape_alloc

pub fn run() {
    crate::serial_println!("[TEST START: tape_alloc]");

    let mut alloc = crate::tape::allocator::BumpAllocator::new(
        crate::tape::regions::KERNEL_HEAP_BASE,
        crate::tape::regions::KERNEL_HEAP_END,
    );

    // Allocate several blocks and verify they don't overlap
    let a1 = alloc.alloc(64, 4).expect("alloc 64 bytes");
    let a2 = alloc.alloc(128, 4).expect("alloc 128 bytes");
    let a3 = alloc.alloc(256, 16).expect("alloc 256 bytes, align 16");

    // Each allocation must start after the previous one ends
    assert!(a2 >= a1 + 64,  "a2 overlaps a1");
    assert!(a3 >= a2 + 128, "a3 overlaps a2");

    // a3 must be 16-byte aligned
    assert_eq!(a3 % 16, 0, "a3 not aligned to 16");

    // All addresses must be within the heap region
    assert!(a1 >= crate::tape::regions::KERNEL_HEAP_BASE);
    assert!(a3 + 256 <= crate::tape::regions::KERNEL_HEAP_END);

    // OOM should return None, not panic
    let huge = alloc.alloc(u32::MAX, 4);
    assert!(huge.is_none(), "OOM did not return None");

    crate::serial_println!("  TAPE: alloc a1={:#010x} a2={:#010x} a3={:#010x}", a1, a2, a3);
    crate::serial_println!("[TEST PASS: tape_alloc]");
    super::head_boot_basic::qemu_exit_success();
}

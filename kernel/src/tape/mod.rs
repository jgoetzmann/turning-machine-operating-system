// kernel/src/tape/mod.rs
// TAPE domain — Physical memory manager (the Turing Machine tape itself)
// Agent: Agent-03
// Read docs/agents/TAPE.md before modifying.

pub mod regions;      // ALL physical address constants — single source of truth
pub mod allocator;    // Bump allocator (startup phase)
pub mod heap;         // Slab heap (runtime, registered as global allocator)

use crate::head::TapeRegion;

/// Initialize the tape: parse memory map, set up allocator, bring up heap.
/// Called by HEAD domain (boot_init) after GDT is loaded.
///
/// Order: regions validated → bump allocator ready → heap initialized
pub fn init_tape(memory_map: &[TapeRegion]) {
    // TODO (Agent-03): implement tape initialization
    // 1. Validate memory map regions against regions.rs constants
    // 2. Initialize bump allocator from KERNEL_HEAP_BASE
    // 3. Call heap::init_heap() to register the global allocator
    let _ = memory_map;
    crate::serial_println!("TAPE: init (TODO — Agent-03)");
}

/// Read a typed value from a physical tape address.
///
/// # Safety
/// Caller must ensure:
/// - `phys` is within a valid tape region (use `is_valid_tape_range` to check)
/// - `phys` is correctly aligned for type T
/// - The region contains initialized data of type T
#[inline]
pub unsafe fn tape_read<T: Copy>(phys: u32) -> T {
    core::ptr::read_volatile(phys as *const T)
}

/// Write a typed value to a physical tape address.
///
/// # Safety
/// Caller must ensure:
/// - `phys` is within a valid, writable tape region
/// - `phys` is correctly aligned for type T
#[inline]
pub unsafe fn tape_write<T: Copy>(phys: u32, val: T) {
    core::ptr::write_volatile(phys as *mut T, val)
}

/// Validate that a physical address range is within a known tape region.
pub fn is_valid_tape_range(base: u32, len: u32) -> bool {
    // TODO (Agent-03): check against all known regions in regions.rs
    let _ = (base, len);
    true // placeholder — replace with real check
}

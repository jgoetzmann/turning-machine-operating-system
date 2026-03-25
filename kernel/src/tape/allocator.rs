// kernel/src/tape/allocator.rs
// Bump allocator — startup phase allocation.
// Agent: TAPE (Agent-03)
//
// The bump allocator is the simplest possible allocator:
// it keeps a pointer (the "head") starting at KERNEL_HEAP_BASE
// and bumps it forward on every allocation. It never frees.
//
// Used during kernel init before the slab heap is ready.
// Once init_heap() is called, use the global allocator instead.

use crate::tape::regions::{KERNEL_HEAP_BASE, KERNEL_HEAP_END};

/// A simple bump allocator that moves rightward along a tape region.
pub struct BumpAllocator {
    head: u32,   // Next free physical address (moves right)
    end:  u32,   // One past the last usable address
}

impl BumpAllocator {
    /// Create a new bump allocator for a tape region.
    pub const fn new(base: u32, end: u32) -> Self {
        Self { head: base, end }
    }

    /// Allocate `size` bytes aligned to `align` bytes.
    /// Returns the physical address of the allocation, or None if out of tape.
    ///
    /// `align` must be a power of 2.
    pub fn alloc(&mut self, size: u32, align: u32) -> Option<u32> {
        // Align the current head upward
        let aligned = align_up(self.head, align);

        // Check for overflow and bounds
        let end = aligned.checked_add(size)?;
        if end > self.end {
            return None;  // Out of tape — no panic, just None
        }

        self.head = end;
        Some(aligned)
    }

    /// How many bytes remain in this allocator's region.
    pub fn remaining(&self) -> u32 {
        self.end.saturating_sub(self.head)
    }

    /// Current head position (next free physical address).
    pub fn current(&self) -> u32 {
        self.head
    }
}

/// Round `addr` up to the nearest multiple of `align` (must be power of 2).
#[inline]
pub fn align_up(addr: u32, align: u32) -> u32 {
    debug_assert!(align.is_power_of_two(), "align must be power of 2");
    (addr + align - 1) & !(align - 1)
}

/// Global bump allocator for kernel startup.
/// Used before init_heap() is called.
/// Protected by a spinlock for the (unlikely) case of early IRQ.
pub static BUMP: spin::Mutex<BumpAllocator> =
    spin::Mutex::new(BumpAllocator::new(KERNEL_HEAP_BASE, KERNEL_HEAP_END));

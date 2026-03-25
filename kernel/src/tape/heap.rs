// kernel/src/tape/heap.rs
// Kernel heap — global allocator backed by the kernel heap tape region.
// Agent: TAPE (Agent-03)
//
// Once init_heap() is called, Rust's Box, Vec, String, Arc, etc. all work.
// The allocator uses the KERNEL_HEAP tape region (1MB at 0x00300000).
//
// Implementation approach:
//   Phase 1: linked-list allocator (simple, correct, no external deps)
//   Phase 2: slab allocator (faster for fixed-size kernel objects)
//
// Do NOT call Box/Vec/Arc before init_heap() is called — it will OOM immediately.

use crate::tape::regions::{KERNEL_HEAP_BASE, KERNEL_HEAP_SIZE};

// TODO (Agent-03): Choose a no_std heap allocator.
// Recommended: linked_list_allocator crate (add to Cargo.toml).
//
// Example integration:
//
//   use linked_list_allocator::LockedHeap;
//   #[global_allocator]
//   static ALLOCATOR: LockedHeap = LockedHeap::empty();
//
//   pub fn init_heap() {
//       let mut allocator = ALLOCATOR.lock();
//       // SAFETY: KERNEL_HEAP_BASE..KERNEL_HEAP_BASE+KERNEL_HEAP_SIZE is valid RAM,
//       //         not used by anything else, and this is called exactly once.
//       unsafe {
//           allocator.init(KERNEL_HEAP_BASE as *mut u8, KERNEL_HEAP_SIZE as usize);
//       }
//       crate::serial_println!("TAPE: heap initialized ({} KB)", KERNEL_HEAP_SIZE / 1024);
//   }
//
// For now, a placeholder that makes the crate compile:

struct PlaceholderAllocator;

unsafe impl core::alloc::GlobalAlloc for PlaceholderAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // TODO (Agent-03): Replace with real heap allocator.
        // For now, use the bump allocator as a "never-free" heap.
        let size  = layout.size() as u32;
        let align = layout.align() as u32;
        match crate::tape::allocator::BUMP.lock().alloc(size, align) {
            Some(phys) => phys as *mut u8,
            None       => core::ptr::null_mut(),  // OOM — triggers alloc_error_handler
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // Bump allocator cannot free — this is a placeholder.
        // Replace with linked-list allocator for real deallocation.
    }
}

#[global_allocator]
static ALLOCATOR: PlaceholderAllocator = PlaceholderAllocator;

/// Initialize the kernel heap.
/// Must be called after bump allocator is available.
/// Enables Box, Vec, Arc, etc. for the rest of the kernel.
pub fn init_heap() {
    // TODO (Agent-03): Initialize the real heap allocator here.
    // See the linked_list_allocator example above.
    crate::serial_println!("TAPE: heap region {:#010x}–{:#010x} ({} KB)",
        KERNEL_HEAP_BASE,
        KERNEL_HEAP_BASE + KERNEL_HEAP_SIZE - 1,
        KERNEL_HEAP_SIZE / 1024
    );
    crate::serial_println!("TAPE: heap init (TODO — Agent-03, use linked_list_allocator)");
}

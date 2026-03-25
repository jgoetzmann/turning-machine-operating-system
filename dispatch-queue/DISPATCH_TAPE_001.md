# DISPATCH_TAPE_001 — Tape Regions + Bump Allocator + Heap

## Header
**Domain:** TAPE | **Agent:** Agent-03 | **Status:** PENDING
**Priority:** CRITICAL | **Created:** PROJECT_START
**Branch:** agent/tape-init

## Prerequisites
- [ ] DISPATCH_HEAD_001 COMPLETE — need `head::boot_init` to call `tape::init_tape()`

## Context Snapshot
None — load `docs/agents/TAPE.md` only.

## Objective
Define all physical tape region constants. Implement a bump allocator for kernel startup. Implement a slab heap (global allocator) so `Box`/`Vec` work in the kernel. After this task, the kernel heap is live and all domains can allocate.

## Acceptance Criteria
- [ ] `tape/regions.rs`: ALL constants from TAPE.md Section 3 defined as `pub const u32`
- [ ] No other file may hardcode a physical address — must use these constants
- [ ] `BumpAllocator` allocates from `KERNEL_HEAP_BASE` upward, returns `u32` physical addr
- [ ] `BumpAllocator` returns `None` when past `KERNEL_HEAP_END` (no panic)
- [ ] `init_heap()` registers slab allocator as `#[global_allocator]`
- [ ] `tape_read<T>()` and `tape_write<T>()` wrappers use `read_volatile`/`write_volatile`
- [ ] `is_valid_tape_range()` checks any `(base, len)` against known regions
- [ ] `make qemu-test TEST=tape_alloc` passes
- [ ] `make qemu-test TEST=tape_regions` passes (sanity check constants)
- [ ] **All addresses are `u32`. No `u64` anywhere in this module.**

## Files To Touch
- CREATE: `kernel/src/tape/regions.rs` — all physical address constants
- CREATE: `kernel/src/tape/allocator.rs` — BumpAllocator
- CREATE: `kernel/src/tape/heap.rs` — slab heap + global allocator registration
- CREATE: `kernel/src/tape/mod.rs` — `pub fn init_tape()`, re-exports
- MODIFY: `kernel/src/head/mod.rs` — call `tape::init_tape(memory_map)` after GDT
- CREATE: `tests/integration/tape_alloc.rs`
- CREATE: `tests/integration/tape_regions.rs`

## Pitfalls
- See MISTAKE-003 (indirectly): `tape_write` must use `write_volatile` — see MISTAKE-007 for VGA
- See TAPE.md Section 6: heap before init_heap panics. Call order: `init_tape` → bump → `init_heap`
- Bump allocator must align allocations (align to 4 bytes minimum for 32-bit)
- Stack grows DOWN from `KERNEL_STACK_TOP = 0x00007000`. Heap grows UP from `KERNEL_HEAP_BASE = 0x00300000`. They do not overlap.

## Test
```bash
make qemu-test TEST=tape_alloc
make qemu-test TEST=tape_regions
```

## On Completion
Generate: DISPATCH_CTRL_001, DISPATCH_PROC_001

---

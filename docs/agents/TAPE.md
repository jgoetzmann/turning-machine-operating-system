# TAPE.md — Memory Manager Agent
> **Domain ID:** `TAPE` | **Agent Slot:** Agent-03
> **TM Role:** The tape itself. Track which cells are used. Allocate regions. Protect boundaries.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

TAPE owns the physical address space — the actual Turing Machine tape.

- **Region registry**: canonical mapping of which physical addresses mean what (from `regions.rs`)
- **Bump allocator**: tape head moves right to allocate — simple, no free list, used for kernel startup
- **Slab heap**: reusable kernel heap after bump phase (for `Box`, `Vec`, etc.)
- **Region guard**: runtime checks that no domain writes outside its tape region
- **Tape read/write API**: typed wrappers around `read_volatile`/`write_volatile`

There is **no virtual memory**. There is **no paging**. There is only the tape.

---

## 2. File Ownership

```
kernel/src/tape/
├── mod.rs              # Public API, init_tape(), global allocator
├── regions.rs          # ALL physical address constants — single source of truth
├── allocator.rs        # Bump allocator (moves tape head rightward)
└── heap.rs             # Slab allocator for kernel heap region
```

---

## 3. Tape Region Constants (`regions.rs`) — Source of Truth

```rust
// kernel/src/tape/regions.rs
// ALL physical addresses live here. No other file may hardcode an address.

pub const TAPE_START:       u32 = 0x0000_0000;
pub const TAPE_END:         u32 = 0x0100_0000;  // 16MB

// Hardware-mapped tape regions
pub const VGA_BASE:         u32 = 0x000B_8000;  // VGA text buffer (80x25 chars)
pub const VGA_END:          u32 = 0x000B_FFFF;

// Kernel tape regions
pub const KERNEL_CODE_BASE: u32 = 0x0010_0000;  // Kernel ELF loaded here
pub const KERNEL_DATA_BASE: u32 = 0x0020_0000;
pub const KERNEL_HEAP_BASE: u32 = 0x0030_0000;  // Slab heap starts here
pub const KERNEL_HEAP_END:  u32 = 0x0040_0000;  // 1MB heap
pub const KERNEL_STACK_TOP: u32 = 0x0007_0000;  // Stack grows DOWN from here

// Process tape
pub const PROC_TAPE_BASE:   u32 = 0x0040_0000;  // Process memory starts here
pub const PROC_TAPE_END:    u32 = 0x0060_0000;  // 2MB per process slot
pub const PROC_STACK_TOP:   u32 = 0x0070_0000;  // User stack top

// Shared IPC tape
pub const IPC_TAPE_BASE:    u32 = 0x0070_0000;
pub const IPC_TAPE_END:     u32 = 0x0080_0000;

// Filesystem cache tape
pub const FS_CACHE_BASE:    u32 = 0x0080_0000;
pub const FS_CACHE_END:     u32 = 0x00C0_0000;
```

---

## 4. API Contracts

```rust
// tape/mod.rs

/// Initialize the tape allocator from the memory map provided by HEAD.
pub fn init_tape(memory_map: &[TapeRegion]);

/// Read a typed value from a physical tape address.
/// SAFETY: caller ensures address is within a valid tape region and aligned.
pub unsafe fn tape_read<T: Copy>(phys: u32) -> T {
    core::ptr::read_volatile(phys as *const T)
}

/// Write a typed value to a physical tape address.
/// SAFETY: caller ensures address is within a valid tape region and aligned.
pub unsafe fn tape_write<T: Copy>(phys: u32, val: T) {
    core::ptr::write_volatile(phys as *mut T, val)
}

/// Validate that a physical address range is within a known tape region.
pub fn is_valid_tape_range(base: u32, len: u32) -> bool;

// tape/allocator.rs (bump allocator — startup only)
pub struct BumpAllocator { head: u32, end: u32 }
impl BumpAllocator {
    pub fn new(base: u32, end: u32) -> Self;
    pub fn alloc(&mut self, size: u32, align: u32) -> Option<u32>;  // returns physical addr
}

// tape/heap.rs (slab heap — runtime)
pub fn init_heap();
// Registered as #[global_allocator] — enables Box, Vec, etc. in kernel
```

---

## 5. Dependencies

| Depends On | What You Need |
|---|---|
| `HEAD` | `head::get_tape_memory_map()` — know which physical regions are usable |

---

## 6. Critical Constraints

- **No paging, no virtual addresses.** Every address is a `u32` physical address. Do not use pointer arithmetic that assumes a higher-half offset.
- **`tape_read` and `tape_write` are the canonical interface.** No domain should use `*ptr` directly for tape-mapped hardware registers — they must go through the volatile wrappers.
- **Bump allocator is for startup only.** Once `init_heap()` is called, use the slab heap. The bump allocator cannot free.
- **VGA buffer is a tape region.** Writes to `VGA_BASE..VGA_END` are `tape_write` calls. DISPLAY domain uses this.
- **Stack grows DOWN** from `KERNEL_STACK_TOP`. Allocating tape upward from `KERNEL_HEAP_BASE` will not collide with the stack.
- **No allocations below `KERNEL_HEAP_BASE`** from user code. The low tape regions (`0x0000`–`0x000FFFFF`) are BIOS/boot territory.
- **Region guard**: add a `debug_assert!(is_valid_tape_range(addr, size))` before any tape_write in debug builds.

---

## 7. The Tape Is The Machine

Unlike a conventional OS where "memory" is an implementation detail, in this OS **the tape is the system**. Every domain has a named tape region. When a domain reads/writes memory it is literally moving the TM head across the tape.

This means:
- Tape corruption = the TM reading garbage symbols = undefined behavior
- Tape bounds = the TM head moving off the tape = machine halts
- Tape regions = sections of the tape with defined alphabets

Keep this in mind when designing APIs. Ask: "what does this tape region contain, and what symbols are valid there?"

---

## 8. Known Mistakes to Avoid

- **MISTAKE: Dereferencing `u32` as pointer without cast** — `let val = *(0xB8000 as *const u16)` — must be `read_volatile`. Plain reads may be optimized away.
- **MISTAKE: Using heap before `init_heap()`** — the global allocator panics if called before `init_heap`. Init order: `init_tape()` → bump alloc → `init_heap()` → slab alloc.
- **MISTAKE: Hardcoding `0xB8000` in DISPLAY code** — must use `tape::regions::VGA_BASE`. If we ever add a test with a different VGA address, this breaks.
- **MISTAKE: Allocating kernel heap into process tape** — bump allocator must not grow past `KERNEL_HEAP_END`. Add a bounds check in `BumpAllocator::alloc`.

---

## 9. Task Workflow

```bash
echo "agent: Agent-03 / task: tape-init / started: $(date)" > task-locks/TAPE_init.lock
make build
make qemu-test TEST=tape_alloc
make qemu-test TEST=tape_regions
git rm task-locks/TAPE_init.lock
```

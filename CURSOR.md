# CURSOR.md — Cursor IDE Rules & AI Context

This file tells Cursor's AI assistant how to help on this project.
The `.cursorrules` file at root is the machine-readable version Cursor reads automatically.

---

## Project Identity

This is a **Turing Machine Operating System** targeting x86 32-bit (i686) protected mode.
The entire OS is designed around the TM model: tape (memory), head (CPU), state machine (kernel), transition function (dispatcher).

**Key facts Cursor must know:**
- Target: `i686-unknown-none` (32-bit, no OS, bare metal)
- Mode: Protected mode, flat segmentation. **NO PAGING.** `CR0.PG` is never set.
- All memory addresses are **physical**. No virtual addresses exist.
- No standard library: `#![no_std]` everywhere in kernel.
- Syscall mechanism: `int 0x80` (32-bit Linux ABI). NOT `syscall` instruction.
- Registers: 32-bit (`eax`, `ebx`, `ecx`, etc). Not `rax`, `rbx`.
- The kernel IS a state machine. State lives in `kernel::ctrl::MACHINE_STATE`.
- All state transitions go through `kernel::ctrl::transition()`.

---

## How Cursor Should Assist

### Code Generation Rules

1. **Always 32-bit Rust**: Use `u32` for addresses, not `u64`. Use `usize` where width-independent.
2. **No paging constructs**: Do not generate page tables, CR3 loads, PML4/PDPT/PD/PT structures. These are forbidden.
3. **Physical addresses only**: When generating memory access code, all pointers are physical addresses.
4. **Every `unsafe` needs `// SAFETY:`**: No exceptions. Cursor must add this comment when generating unsafe code.
5. **No `std`**: If Cursor would normally use `std::`, find the `core::` equivalent or suggest a `no_std` crate.
6. **Port I/O is via inline ASM or `x86` crate**: Use `x86::io::inb/outb` or `asm!("in/out ...")`.
7. **Interrupt handlers use `extern "x86-interrupt"`**: Only for IDT handlers.
8. **State machine discipline**: Any code that changes machine state must call `ctrl::transition()`, not set `MACHINE_STATE` directly.

### Architecture Reminders Cursor Should Surface

- Memory region constants live in `kernel/src/tape/regions.rs` — never hardcode an address
- The tape layout is fixed (see MASTER.md Section 5)
- VGA buffer is at physical `0x000B8000`
- Kernel loads at physical `0x00100000`
- COM1 serial: base port `0x3F8`
- PIC1: `0x20/0x21`, PIC2: `0xA0/0xA1`
- PIT channel 0: `0x40`, command: `0x43`
- PS/2 keyboard data: `0x60`, status: `0x64`

### What Cursor Should NOT Suggest

- 64-bit constructs (`u64` addresses, `rax`, `EFER` MSR, long mode enable sequence)
- Virtual memory / paging (`cr3`, page tables, TLB flushes, `invlpg`)
- SMP / multi-core (`APIC`, IPI, `lock` prefix for atomic ops across cores)
- Network stack (explicitly out of scope)
- `std::` anything in kernel code
- Dynamic linking or position-independent code in kernel
- Future-proofing abstractions — keep it simple and direct

### Cursor's Suggested Crates (approved for this project)

```toml
spin = "0.9"          # Mutex for single-core (no OS needed)
bitflags = "2"        # Bit manipulation
log = { version = "0.4", default-features = false }
# x86 crate for port I/O (optional but useful)
x86 = { version = "0.52", default-features = false }
```

---

## Domain Quick Reference (for Cursor)

When editing a file, Cursor should know which domain owns it:

| Path | Domain | Agent |
|---|---|---|
| `kernel/src/head/` | HEAD | Agent-02 |
| `kernel/src/tape/` | TAPE | Agent-03 |
| `kernel/src/ctrl/` | CTRL | Agent-04 |
| `kernel/src/trans/` | TRANS | Agent-05 |
| `kernel/src/device/` | DEVICE | Agent-06 |
| `kernel/src/display/` | DISPLAY | Agent-07 |
| `kernel/src/store/` | STORE | Agent-08 |
| `kernel/src/proc/` | PROC | Agent-09 |
| `userspace/` | SHELL | Agent-10 |
| `tests/` | TEST | Agent-11 |
| `Makefile`, `tools/`, `.cargo/` | BUILD | Agent-01 |

---

## Common Patterns Cursor Should Know

### Reading from a tape region
```rust
// All tape reads are physical address dereferences
// SAFETY: Address is within a valid tape region (see tape::regions)
let value = unsafe { core::ptr::read_volatile(tape::regions::VGA_BASE as *const u16) };
```

### Writing to a tape region
```rust
// SAFETY: VGA_BASE is the fixed tape region for display output (0xB8000)
unsafe { core::ptr::write_volatile((tape::regions::VGA_BASE + offset) as *mut u16, cell) };
```

### Emitting a port I/O (32-bit)
```rust
// SAFETY: Port 0x20 is PIC1 command register; outb is safe for this port
unsafe { x86::io::outb(0x20, 0x20); }  // PIC EOI
```

### Kernel state transition
```rust
// Always go through transition(), never set MACHINE_STATE directly
let next = ctrl::transition(ctrl::current_state(), Event::TimerTick);
ctrl::set_state(next);
```

### Syscall handler signature (32-bit int 0x80)
```rust
// Registers on entry: eax=syscall_num, ebx=arg1, ecx=arg2, edx=arg3
// Return value in eax
pub extern "C" fn handle_syscall(regs: &mut SyscallRegs) -> u32 { ... }
```

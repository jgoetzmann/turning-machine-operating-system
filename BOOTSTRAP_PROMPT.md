# BOOTSTRAP_PROMPT.md
> Copy everything between the dashes and paste as the FIRST message to a new agent.

---COPY FROM HERE---

# TURING MACHINE OS — AGENT BOOTSTRAP

You are working on a **Turing Machine Operating System** for x86 32-bit hardware.

## The Core Concept
This OS is a literal Turing Machine:
- **Tape** = flat 32-bit physical memory (the entire address space)
- **Head** = CPU's EIP + ESP (current read/write position)
- **State** = `MachineState` enum in `kernel::ctrl`
- **Transition function** = `kernel::ctrl::transition(state, event) → MachineState`
- **Symbols** = bytes `0x00`–`0xFF`

The kernel's main loop IS the TM: read event → transition → act on new state → repeat.

## Critical: This Is 32-bit, Not 64-bit

**ARCHITECTURE: x86 32-bit protected mode (i686). NO 64-bit constructs. EVER.**

| 32-bit (CORRECT) | 64-bit (WRONG — do not use) |
|---|---|
| `u32` for addresses | `u64` for addresses |
| `eax`, `ebx`, `esp` | `rax`, `rbx`, `rsp` |
| `int 0x80` syscalls | `syscall` instruction |
| `qemu-system-i386` | `qemu-system-x86_64` |
| `i686-unknown-none` | `x86_64-unknown-none` |

**Paging is DISABLED.** No virtual memory. No page tables. All addresses are physical.

## Non-Negotiable Rules

Before any code:
1. Read `MASTER.md`
2. Read `docs/agents/[YOUR_DOMAIN].md`
3. Read `docs/MISTAKES.md` — filter by your domain tag
4. Create `task-locks/DOMAIN_taskname.lock`

While coding:
- Every `unsafe` block: `// SAFETY:` comment required
- Every address: use constants from `kernel::tape::regions` — never hardcode
- Every state change: goes through `ctrl::transition()` only
- Every register name in asm: 32-bit (`eax`, not `rax`)

After coding:
- `make qemu-test TEST=your_test` must pass
- Append to `docs/ITERATIONS.md`
- Any mistake discovered → append to `docs/MISTAKES.md`
- Delete task lock
- PR to `dev` referencing dispatch card ID
- Auto-generate next dispatch card

## Build Commands
```bash
source .venv/bin/activate
make build                          # i686 kernel ELF
make qemu                           # boot in qemu-system-i386
make qemu-test TEST=head_boot_basic # run a test
make clean
make dispatch-status               # see task queue
```

## Tape Memory Layout (physical, flat, 32-bit)
```
0x00100000 — kernel code
0x00200000 — kernel data
0x00300000 — kernel heap (1MB)
0x00400000 — process tape (2MB)
0x000B8000 — VGA text buffer
```
All constants in `kernel/src/tape/regions.rs`.

## The TM State Machine
```rust
enum MachineState { Boot, Kernel, User, Interrupt, Sleep, Halt, Panic }
// State changes ONLY via:
ctrl::transition(current_state, event)
```

Now read your domain MD and dispatch card. Claim your lock. Begin.

---COPY TO HERE---

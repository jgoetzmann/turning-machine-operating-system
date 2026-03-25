## TMOS in one page

TMOS (“Turing Machine OS”) is a from-scratch **32-bit x86 operating system** designed as a **literal Turing Machine**:

- **Tape**: flat 32-bit physical memory (no virtual memory)
- **Head**: CPU execution position (primarily `EIP` + `ESP`)
- **State register**: `kernel::ctrl::MACHINE_STATE`
- **Transition function**: `kernel::ctrl::transition(state, event) -> next_state`

The project’s “north star” is stated in `MASTER.md`: everything else (drivers, filesystem, shell) exists to serve the transition function.

## Non-negotiable specs / constraints

These are enforced repeatedly across `MASTER.md`, `CURSOR.md`, and `CONTRIBUTING.md`:

- **32-bit only**:
  - Target: `i686-unknown-none`
  - QEMU: `qemu-system-i386` (never `x86_64`)
  - Syscalls: `int 0x80` (never the x86_64 `syscall` instruction)
  - Addresses: use `u32` for physical addresses
- **No paging / no virtual memory / no higher-half**:
  - Physical addresses only; `CR0.PG` is never set.
- **Single core**: one head, one CPU; no SMP design.
- **Tape layout is locked**:
  - All physical addresses come from `kernel/src/tape/regions.rs` (INV-03).
- **`#![no_std]` kernel**:
  - No `std` anywhere under `kernel/`.
- **Safety rule**:
  - Every `unsafe` block must include a `// SAFETY:` justification.

## Repo structure (domains)

TMOS is organized into “domains” (ownership boundaries). The major paths:

- **Kernel**: `kernel/src/`
  - `head/`: boot + GDT/IDT + serial + multiboot memmap parsing
  - `tape/`: memory regions + allocator/heap
  - `ctrl/`: machine state + scheduler + machine loop
  - `trans/`: interrupts/exceptions/syscalls (handlers + syscall gate)
  - `device/`: PIC/PIT/keyboard/ATA drivers
  - `display/`: VGA text output
  - `store/`: FAT12 (and higher-level storage APIs)
  - `proc/`: process model + context switching
- **Userspace**: `userspace/` (shell + libc syscall wrappers)
- **Tests**: `tests/` (QEMU integration tests compiled into the kernel)
- **Tooling**: `tools/` (Python helpers: test runner, dispatch generator, context snapshots)

For the authoritative map (including workflow folders like `dispatch-queue/` and `task-locks/`), see `MASTER.md` §7.

## “Format” of work in this repo (dispatch + locks + logs)

This repository is intentionally structured for multi-task / multi-agent work:

- **Dispatch cards**: task specs live in `dispatch-queue/DISPATCH_<DOMAIN>_<NNN>.md`
- **Task locks**: task claims live in `task-locks/<DOMAIN>_<task>.lock`
- **Append-only logs**:
  - `docs/ITERATIONS.md`: what changed and why (append-only)
  - `docs/MISTAKES.md`: known pitfalls with prevention rules (append-only)

If you’re contributing: read `docs/AGENT_WORKFLOW.md` and `docs/TASK_DISPATCH.md` before opening a PR.

## How to build, run, and test (local)

### Setup

```bash
# Python tooling (optional, but needed for the test runner and dispatch tools)
bash tools/setup_venv.sh
source .venv/bin/activate
```

You’ll also need **Rust nightly**, **QEMU**, and **NASM** (see `README.md`).

### Build + verify architecture

```bash
make build
make verify-32bit
```

### Run

```bash
make qemu
```

### Run tests (QEMU integration harness)

```bash
make test-list
make qemu-test TEST=head_boot_basic
make qemu-test-all
```

Tests are selected by passing `-append "test=..."` to QEMU; the kernel’s `kernel/src/test_dispatch.rs` routes to the requested test and exits QEMU via `isa-debug-exit`.

## Where execution starts (boot → machine loop)

The concrete boot path is:

- `kernel/src/head/boot.asm` (`_start`) sets a stack, zeros `.bss`, calls Rust `boot_init`
- `kernel/src/head/mod.rs` (`boot_init`) initializes serial, validates multiboot, sets up GDT/IDT skeleton, parses memory map, then calls:
- `kernel/src/ctrl/mod.rs` (`machine_boot`) which initializes devices + tape and enters:
- `kernel/src/ctrl/mod.rs` (`machine_loop`) forever

## Important “current state” notes (practical reality)

The design docs are strict; the implementation is in-progress. A few high-signal realities today:

- **`ctrl::push_event()` is currently a stub** (events are dropped), so IRQ/syscall-driven state transitions aren’t wired end-to-end yet.
- **`machine_boot()` currently enables interrupts (`sti`) but has a TODO where handler registration should occur**. Until `trans::register_all_handlers()` is called before `sti`, any IRQ can lead to faults/reset (see the warning in `head/idt.rs`).

These are excellent “first tasks” if you want to help safely, because they’re small, testable, and unblock other work.

## Good next steps (pick a lane)

If you want to contribute, here are safe, high-leverage next steps:

- **Get green locally**:
  - `make build && make verify-32bit && make qemu-test TEST=head_boot_basic`
- **Pick one small dispatch-card-sized improvement**:
  - Wire `trans::register_all_handlers()` into `ctrl::machine_boot()` *before* `sti`
  - Implement a minimal `ctrl` event queue (fixed-size ring buffer) so IRQs can post events safely
  - Expand/strengthen one QEMU integration test to guard an invariant (PIC EOI, timer ticks, VGA writes)
- **Follow the repo’s “format”**:
  - Create/claim a task lock, work on a branch named `agent/<DOMAIN>-<task>`, update `docs/ITERATIONS.md`, delete the lock in the PR.


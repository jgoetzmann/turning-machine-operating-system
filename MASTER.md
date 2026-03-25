# MASTER.md — Turing Machine OS
> **ALL AGENTS READ THIS FIRST. NO EXCEPTIONS.**
> Then: your domain MD → `docs/MISTAKES.md` → `docs/AGENT_WORKFLOW.md` → start work.

---

## 1. The Core Thesis

This OS is a **literal Turing Machine** running on x86 32-bit hardware.

Every classical TM component maps directly to a real system component:

| Turing Machine | This OS |
|---|---|
| **Tape** | Flat 32-bit physical memory (the entire address space) |
| **Head** | The CPU's EIP + ESP (read/write position on tape) |
| **Alphabet** | Bytes `0x00`–`0xFF` |
| **Blank symbol** | `0x00` (uninitialized memory) |
| **State register** | `kernel::ctrl::MACHINE_STATE` (a global enum) |
| **Transition function** | `kernel::ctrl::transition(state, symbol) → (new_state, write, direction)` |
| **Accept state** | `MachineState::Halt` (clean shutdown) |
| **Reject state** | `MachineState::Panic` (kernel panic) |

The kernel's main loop is:

```rust
loop {
    let symbol = tape::read(head::position());
    let (next_state, write_symbol, direction) = ctrl::transition(STATE, symbol);
    tape::write(head::position(), write_symbol);
    head::move(direction);
    ctrl::set_state(next_state);
    if STATE == MachineState::Halt { break; }
}
```

Everything else — drivers, filesystem, shell — is an implementation detail of the transition function.

---

## 2. What We Are NOT Building

This project is deliberately minimal. Do not add these:

- ❌ 64-bit long mode (x32 protected mode is the target)
- ❌ Multi-core / SMP (one head, one core)
- ❌ Virtual memory / paging (flat segmentation only)
- ❌ Dynamic linking
- ❌ Network stack
- ❌ UEFI boot (BIOS/Multiboot2 only)
- ❌ Future-proofing abstractions
- ❌ Anything that doesn't serve the TM model

If you find yourself designing for "future expansion," stop. Build the simplest thing that works.

---

## 3. Hardware Target

| Property | Value |
|---|---|
| Architecture | x86 32-bit (i686) |
| Mode | Protected mode, flat segmentation |
| Memory | Flat 32-bit address space, ≤ 128MB assumed |
| Cores | 1 (uniprocessor only) |
| Boot | Multiboot2 via GRUB / QEMU `-kernel` |
| Paging | **DISABLED** — flat segmented model only |
| FPU | Not required (kernel is integer-only) |
| Test platform | QEMU `qemu-system-i386` |
| Primary language | Rust (`no_std`, 32-bit target `i686-unknown-none`) |
| Assembly | NASM (32-bit protected mode) |

---

## 4. Machine States (The State Register)

```rust
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MachineState {
    Boot      = 0x00,  // Initializing tape regions, setting up head
    Kernel    = 0x01,  // Kernel main loop — reading tape, dispatching
    User      = 0x02,  // Executing userspace process on tape
    Interrupt = 0x03,  // Head interrupted — saving position, handling event
    Sleep     = 0x04,  // Waiting for tape input (idle)
    Halt      = 0xFE,  // Accept: clean shutdown
    Panic     = 0xFF,  // Reject: unrecoverable error
}
```

**State transition rules (high level):**

```
Boot      → Kernel    (tape initialized, head positioned at kernel entry)
Kernel    → User      (process loaded onto tape, head transferred)
Kernel    → Sleep     (no runnable process, waiting for tape input)
Kernel    → Halt      (shutdown syscall received)
Kernel    → Panic     (invariant violation)
User      → Kernel    (syscall: head returns to kernel tape region)
User      → Interrupt (IRQ fired during user execution)
Interrupt → User      (IRQ handled, head returns to saved user position)
Interrupt → Kernel    (timer tick: scheduler runs)
Sleep     → Kernel    (tape input arrived, e.g. keyboard)
Panic     → (terminal: print panic tape dump, halt CPU)
```

---

## 5. The Tape Layout (32-bit Address Space)

This is the single tape. All addresses are physical (no virtual memory).

```
Physical Address         Region Name        Domain    Size
────────────────────────────────────────────────────────────
0x00000000 – 0x000004FF  Real Mode IVT      BUILD     (don't touch — BIOS)
0x00000500 – 0x00007BFF  Boot scratch       HEAD      ~30KB
0x00007C00 – 0x00007DFF  Bootloader load    HEAD      512B (MBR zone)
0x00007E00 – 0x0009FFFF  Usable low mem     TAPE      ~620KB
0x000A0000 – 0x000BFFFF  VGA framebuffer    DISPLAY   128KB
0x000C0000 – 0x000FFFFF  BIOS/ROM           (off-limits)
0x00100000 – 0x001FFFFF  Kernel code        HEAD/CTRL 1MB
0x00200000 – 0x002FFFFF  Kernel data + BSS  CTRL/TAPE 1MB
0x00300000 – 0x003FFFFF  Kernel heap        TAPE      1MB
0x00400000 – 0x005FFFFF  Process tape       PROC      2MB per slot
0x00600000 – 0x006FFFFF  Process stack      PROC      1MB
0x00700000 – 0x007FFFFF  Shared tape region IPC       1MB
0x00800000 – 0x00BFFFFF  Filesystem cache   STORE     4MB
0x00C00000 – 0x00FFFFFF  Extended/reserved  TAPE      4MB
────────────────────────────────────────────────────────────
TAPE_END   = 0x01000000  (16MB total addressable, enough for a TM OS)
```

**This layout is locked. See INV-03.**

All regions are defined as constants in `kernel/src/tape/regions.rs`.

---

## 6. Domain Map

| Domain ID | Name | Tape Metaphor | Source Path | Agent |
|---|---|---|---|---|
| `BUILD` | Toolchain | Setup the tape machine | `Makefile`, `tools/`, `.cargo/` | Agent-01 |
| `HEAD` | Boot + CPU Init | Position the head, begin reading | `kernel/src/head/` | Agent-02 |
| `TAPE` | Memory Manager | The tape itself | `kernel/src/tape/` | Agent-03 |
| `CTRL` | State Machine + Scheduler | The finite state control | `kernel/src/ctrl/` | Agent-04 |
| `TRANS` | Interrupts + Syscalls | The transition function | `kernel/src/trans/` | Agent-05 |
| `DEVICE` | Drivers (keyboard, serial, timer) | Tape input/output devices | `kernel/src/device/` | Agent-06 |
| `DISPLAY` | VGA text output | Write symbols to screen tape | `kernel/src/display/` | Agent-07 |
| `STORE` | Filesystem (FAT12) | Persistent tape (disk) | `kernel/src/store/` | Agent-08 |
| `PROC` | Process model | Sub-tape + saved head state | `kernel/src/proc/` | Agent-09 |
| `SHELL` | Userspace + shell | Programs that run on the tape | `userspace/` | Agent-10 |
| `TEST` | Test harness | Verify tape machine behavior | `tests/` | Agent-11 |

---

## 7. Repository Structure (ENFORCE EXACTLY)

```
/
├── MASTER.md                        ← you are here
├── CURSOR.md                        ← Cursor IDE rules
├── .cursorrules                     ← Cursor AI context (auto-read by Cursor)
├── BOOTSTRAP_PROMPT.md              ← paste to orient a new agent
├── CONTRIBUTING.md                  ← PR + code rules
├── README.md
├── Makefile
├── .gitignore
│
├── .cargo/
│   └── config.toml                  ← i686-unknown-none target
│
├── docs/
│   ├── ITERATIONS.md                ← append-only task log
│   ├── MISTAKES.md                  ← append-only mistake registry
│   ├── CONTEXT_PROTOCOL.md          ← context compression rules
│   ├── AGENT_WORKFLOW.md            ← multi-agent coordination
│   ├── TASK_DISPATCH.md             ← dispatch card system
│   └── agents/                      ← one MD per domain
│       ├── BUILD.md
│       ├── HEAD.md
│       ├── TAPE.md
│       ├── CTRL.md
│       ├── TRANS.md
│       ├── DEVICE.md
│       ├── DISPLAY.md
│       ├── STORE.md
│       ├── PROC.md
│       ├── SHELL.md
│       └── TEST.md
│
├── context-snapshots/               ← agent context archives
│   └── YYYY-MM-DD_DOMAIN_vN.md
│
├── dispatch-queue/                  ← pending + active dispatch cards
│   └── DISPATCH_DOMAIN_NNN.md
│
├── task-locks/                      ← active task claims
│   └── DOMAIN_taskname.lock
│
├── kernel/                          ← the TM kernel (no_std Rust)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── linker.ld                    ← flat 32-bit linker script
│   └── src/
│       ├── main.rs                  ← _start → boot_init → machine_loop
│       ├── head/                    ← boot, GDT, IDT setup (HEAD domain)
│       │   ├── mod.rs
│       │   ├── boot.asm             ← multiboot2 header, protected mode entry
│       │   ├── gdt.rs               ← flat 32-bit segments
│       │   ├── idt.rs               ← 32-bit IDT
│       │   └── serial.rs            ← COM1 early debug output
│       ├── tape/                    ← memory manager (TAPE domain)
│       │   ├── mod.rs
│       │   ├── regions.rs           ← ALL tape region constants (single source of truth)
│       │   ├── allocator.rs         ← bump allocator (tape head moves right)
│       │   └── heap.rs              ← kernel heap slab
│       ├── ctrl/                    ← state machine (CTRL domain)
│       │   ├── mod.rs
│       │   ├── state.rs             ← MachineState enum
│       │   ├── transition.rs        ← transition(state, event) → next_state
│       │   └── scheduler.rs         ← pick next process (round robin, 1 at a time)
│       ├── trans/                   ← transition function impl (TRANS domain)
│       │   ├── mod.rs
│       │   ├── syscall.rs           ← syscall dispatch table
│       │   ├── irq.rs               ← IRQ handlers (timer, keyboard)
│       │   └── exceptions.rs        ← #GP, #PF, #DF handlers → Panic state
│       ├── device/                  ← hardware I/O (DEVICE domain)
│       │   ├── mod.rs
│       │   ├── pic.rs               ← 8259 PIC
│       │   ├── pit.rs               ← PIT timer (system tick)
│       │   ├── keyboard.rs          ← PS/2 keyboard → tape input
│       │   └── serial.rs            ← 16550 UART
│       ├── display/                 ← VGA text (DISPLAY domain)
│       │   ├── mod.rs
│       │   └── vga.rs               ← write to 0xB8000 tape region
│       ├── store/                   ← FAT12 filesystem (STORE domain)
│       │   ├── mod.rs
│       │   ├── fat12.rs             ← FAT12 read/write
│       │   └── ata.rs               ← ATA PIO disk driver
│       └── proc/                    ← process model (PROC domain)
│           ├── mod.rs
│           ├── tcb.rs               ← Tape Control Block (process state)
│           └── context.rs           ← save/restore head position (context switch)
│
├── userspace/                       ← programs that run on the tape (SHELL domain)
│   ├── shell/
│   │   └── main.c
│   └── libc/
│       └── syscall.asm              ← raw 32-bit int 0x80 wrappers
│
├── tests/
│   ├── integration/
│   └── fixtures/
│       └── fat12.img
│
└── tools/                           ← Python tooling (.venv only)
    ├── requirements.txt
    ├── setup_venv.sh
    ├── test_runner.py
    ├── dispatch_generator.py
    └── context_compressor.py
```

---

## 8. Global Invariants (NEVER VIOLATE)

| ID | Invariant |
|---|---|
| INV-01 | `#![no_std]` always. No standard library in kernel. |
| INV-02 | Every `unsafe` block has a `// SAFETY:` comment. |
| INV-03 | Tape layout in `kernel/src/tape/regions.rs` is the single source of truth. No hardcoded addresses anywhere else. |
| INV-04 | Paging is DISABLED. All addresses are physical. Never enable CR0.PG. |
| INV-05 | One CPU, one head. No atomic multi-core primitives needed. `spin::Mutex` is fine. |
| INV-06 | The state machine transition function is the ONLY place machine state changes. Never set `MACHINE_STATE` directly outside `ctrl::transition()`. |
| INV-07 | QEMU boot test passes before any merge to `main`. |
| INV-08 | Python lives in `.venv/` and `tools/` only. Never in kernel. |
| INV-09 | FAT12 is the filesystem. Not FAT16. Not FAT32. FAT12 — it fits on a floppy and matches TM simplicity. |
| INV-10 | Userspace calls kernel via `int 0x80` (Linux i386 ABI). Not `syscall` (that's 64-bit). |
| INV-11 | No process preemption in Phase 1 — cooperative multitasking only. The head yields voluntarily. |
| INV-12 | `task-locks/` and `docs/MISTAKES.md` and `docs/ITERATIONS.md` are the agent coordination layer. Use them. |

---

## 9. The Transition Function (Central Design)

The core kernel function. Lives in `kernel/src/ctrl/transition.rs`.

```rust
/// The Turing Machine transition function.
/// Maps (current state, incoming event) → (next state).
/// ALL state changes flow through here.
pub fn transition(state: MachineState, event: Event) -> MachineState {
    match (state, event) {
        (MachineState::Boot,      Event::TapeReady)      => MachineState::Kernel,
        (MachineState::Kernel,    Event::ProcessReady)   => MachineState::User,
        (MachineState::Kernel,    Event::NoProcess)       => MachineState::Sleep,
        (MachineState::Kernel,    Event::Shutdown)        => MachineState::Halt,
        (MachineState::User,      Event::Syscall(n))      => handle_syscall(n),
        (MachineState::User,      Event::Irq(n))          => MachineState::Interrupt,
        (MachineState::Interrupt, Event::IrqDone)         => MachineState::User,
        (MachineState::Interrupt, Event::TimerTick)       => MachineState::Kernel,
        (MachineState::Sleep,     Event::TapeInput)       => MachineState::Kernel,
        (_,                       Event::Fault(_))        => MachineState::Panic,
        (s, e) => { serial_println!("UNHANDLED: {:?} + {:?}", s, e); MachineState::Panic }
    }
}
```

This is not metaphorical. This is the actual kernel dispatch.

---

## 10. Agent Onboarding Checklist

Complete IN ORDER before writing a single line:

- [ ] Read MASTER.md (this file)
- [ ] Read `docs/agents/[YOUR_DOMAIN].md`
- [ ] Read `docs/MISTAKES.md` — filter by your domain tag
- [ ] Read `docs/ITERATIONS.md` — last 3 entries
- [ ] Read `docs/AGENT_WORKFLOW.md`
- [ ] Check `task-locks/` — is your task claimed?
- [ ] Create `task-locks/DOMAIN_taskname.lock`
- [ ] Begin work

---

## 11. Build Order (Dependency Graph)

```
BUILD (toolchain) → required by ALL
HEAD  → depends on BUILD
TAPE  → depends on HEAD
CTRL  → depends on TAPE, HEAD
TRANS → depends on CTRL, DEVICE
DEVICE→ depends on HEAD, TAPE
DISPLAY→ depends on HEAD (VGA address is fixed tape region)
STORE → depends on TAPE, DEVICE
PROC  → depends on TAPE, CTRL
SHELL → depends on PROC, TRANS, STORE, DISPLAY
TEST  → tests everything
```

---

## 12. Git Strategy

| Branch | Purpose |
|---|---|
| `main` | Stable, QEMU-tested only |
| `dev` | Integration — domains merge here |
| `agent/DOMAIN-task` | Active work |
| `fix/DOMAIN-MISTAKE-NNN` | Bug fixes |

Every PR references a dispatch card ID. No exceptions.

# Turing Machine OS

A from-scratch operating system built as a **literal Turing Machine** on x86 32-bit hardware.

```
Tape   = flat 32-bit physical memory
Head   = CPU's EIP + ESP
States = Boot → Kernel → User → Interrupt → Sleep → Halt / Panic
δ      = kernel::ctrl::transition(state, event) → next_state
```

## Quick Start

```bash
# 1. Install Rust nightly
rustup toolchain install nightly
rustup component add rust-src

# 2. Install QEMU (32-bit)
# Ubuntu:  sudo apt install qemu-system-x86
# macOS:   brew install qemu

# 3. Install NASM
# Ubuntu:  sudo apt install nasm
# macOS:   brew install nasm

# 4. Python tooling
bash tools/setup_venv.sh
source .venv/bin/activate

# 5. Build (verify it's 32-bit!)
make build
make verify-32bit    # must say "Intel 80386"

# 6. Run
make qemu            # uses qemu-system-i386 (NOT x86_64)

# 7. Test
make qemu-test TEST=head_boot_basic
make qemu-test-all
```

## Dev conveniences (venv + smoke)

If you want a repeatable local setup for the repo’s Python tooling:

```bash
bash scripts/setup_venv.sh
source .venv/bin/activate
```

To run a small “smoke” sequence (build → verify 32-bit → a couple QEMU tests → compile tiny freestanding C programs):

```bash
bash scripts/smoke.sh
```

### Docker (avoid global installs)

If you’d rather avoid installing Rust/QEMU/NASM globally, you can run the same smoke suite inside a container:

```bash
bash scripts/docker_smoke.sh
```

## Architecture

```
┌─────────────────────────────────────┐
│  USERSPACE (SHELL)                  │ ← programs on process tape
│  int 0x80 syscalls                  │
├─────────────────────────────────────┤
│  TRANS: transition function impl    │ ← syscall + IRQ handlers
│  CTRL:  state machine + scheduler   │ ← finite state control
├──────────┬──────────┬───────────────┤
│  PROC    │  STORE   │  DISPLAY      │
│  DEVICE  │  TAPE    │               │
├─────────────────────────────────────┤
│  HEAD: boot, GDT, IDT               │ ← position the head
├─────────────────────────────────────┤
│  x86 32-bit hardware (i686)         │ ← the tape machine
└─────────────────────────────────────┘
```

## ⚠️ This Is 32-bit, Not 64-bit

- Target: `i686-unknown-none`
- QEMU: `qemu-system-i386`
- Syscalls: `int 0x80` (NOT `syscall`)
- Addresses: `u32` (NOT `u64`)
- No paging, no virtual memory, no higher-half

## Documentation

| File | Purpose |
|---|---|
| `MASTER.md` | Project north star — read first |
| `docs/START_HERE.md` | One-page “what is TMOS + how to build/test + next steps” |
| `CURSOR.md` | Cursor IDE rules |
| `.cursorrules` | Auto-read by Cursor AI |
| `BOOTSTRAP_PROMPT.md` | Paste to orient a new agent |
| `docs/agents/` | Per-domain specs (11 domains) |
| `docs/MISTAKES.md` | Known mistakes — read before coding |
| `docs/ITERATIONS.md` | Task history |
| `docs/AGENT_WORKFLOW.md` | Multi-agent coordination |
| `docs/TASK_DISPATCH.md` | Dispatch card system |
| `docs/CONTEXT_PROTOCOL.md` | Context compression |
| `CONTRIBUTING.md` | PR rules and code standards |

## For Agents

Read `MASTER.md` → your `docs/agents/DOMAIN.md` → `docs/MISTAKES.md` → claim task lock → work.

See `BOOTSTRAP_PROMPT.md` for the full agent orientation prompt.

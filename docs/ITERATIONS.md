# ITERATIONS.md — Task Log
> **APPEND-ONLY. Fill one entry per task completed. Never delete.**

---

## Schema

```
## [ITER-NNN] [DOMAIN] — Title
**Date:** YYYY-MM-DD | **Agent:** Agent-XX | **Status:** OPEN | SUCCESS | FAILED | ABANDONED

### Goal
What you were trying to do.

### Approach
What you tried.

### Result
What happened.

### What Failed (if applicable)
Exact failure + root cause.

### Decisions Locked
- LOCKED: decision — reason

### Mistakes Logged
→ MISTAKE-NNN (if any added)

### Next Step
One sentence — the natural next task.

### Dispatch Cards Generated
→ DISPATCH_DOMAIN_NNN
---
```

---

## [ITER-001] [BUILD] — Bootstrap 32-bit build system
**Date:** PROJECT_START | **Agent:** Agent-01 | **Status:** OPEN

### Goal
Set up `i686-unknown-none` Rust cross-compile target, Makefile, Python venv. `make build` produces a bootable 32-bit ELF.

### Approach
(Agent-01 fills this in)

### Result
(Agent-01 fills this in)

### What Failed
(Agent-01 fills this in)

### Decisions Locked
- LOCKED: Target is `i686-unknown-none` (32-bit bare metal, not 64-bit)
- LOCKED: Python venv at `.venv/`, never committed
- LOCKED: QEMU binary is `qemu-system-i386`, not `qemu-system-x86_64`

### Mistakes Logged
(none yet)

### Next Step
HEAD: implement multiboot2 header + serial hello world.

### Dispatch Cards Generated
→ DISPATCH_HEAD_001

---

## [ITER-002] [HEAD] — Multiboot2 entry + serial output
**Date:** TBD | **Agent:** Agent-02 | **Status:** OPEN

### Goal
GRUB/QEMU loads kernel. CPU in 32-bit protected mode. GDT loaded. Serial prints "HEAD: tape machine booting". Test passes.

### Approach
(Agent-02 fills this in)

### Result
(Agent-02 fills this in)

### What Failed
(Agent-02 fills this in)

### Decisions Locked
(Agent-02 fills this in)

### Mistakes Logged
(Agent-02 fills this in)

### Next Step
TAPE: initialize regions, bump allocator.

### Dispatch Cards Generated
→ DISPATCH_TAPE_001
→ DISPATCH_DEVICE_001 (can start in parallel with TAPE)

---

*— Append all subsequent iterations below —*

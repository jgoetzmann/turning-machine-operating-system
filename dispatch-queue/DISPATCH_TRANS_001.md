# DISPATCH_TRANS_001 — Syscall Gate + IRQ Handlers

## Header
**Domain:** TRANS | **Agent:** Agent-05 | **Status:** PENDING
**Priority:** HIGH | **Created:** PROJECT_START
**Branch:** agent/trans-syscall-irq

## Prerequisites
- [ ] DISPATCH_CTRL_001 COMPLETE (needs Event types and push_event)
- [ ] DISPATCH_DEVICE_001 COMPLETE (PIC must be set up before wiring IRQ handlers)

## Context Snapshot
None — load `docs/agents/TRANS.md` only.

## Objective
Wire up all IDT gates: `int 0x80` syscall entry (DPL=3), IRQ0 timer, IRQ1 keyboard, and all exception handlers. After this task, userspace can call `int 0x80` and return to userspace, and the PIT fires timer events into the CTRL event queue.

## Acceptance Criteria
- [ ] `int 0x80` IDT gate: DPL=3, points to `syscall_entry` naked function
- [ ] `syscall_entry` saves all registers with `pushad`, calls `syscall_dispatch`, restores, `iret`
- [ ] `syscall_dispatch(num, ebx, ecx, edx)` matches Phase 1 syscalls: EXIT, WRITE, GETPID, YIELD
- [ ] Exception handlers (#GP, #DF, #PF) push `Event::Fault` and do NOT return
- [ ] IRQ0 handler: increments tick counter, sends EOI, pushes `Event::TimerTick`
- [ ] IRQ1 handler: reads scancode, sends EOI, pushes `Event::TapeInput`
- [ ] `make qemu-test TEST=trans_syscall` passes (userspace `int 0x80` returns correctly)
- [ ] `make qemu-test TEST=trans_timer` passes (timer events fire at ~100Hz)

## Pitfalls
- See MISTAKE-003: Every IRQ handler MUST send PIC EOI as last step before iret
- See MISTAKE-004: Syscall IDT gate MUST be DPL=3
- See TRANS.md: `pushad` saves 8 registers in order. `popad` restores in reverse.
- 32-bit `iret` pops: EIP, CS, EFLAGS (and ESP, SS if privilege change)

## Test
```bash
make qemu-test TEST=trans_syscall
make qemu-test TEST=trans_timer
```

## On Completion
Generate: DISPATCH_PROC_001, DISPATCH_STORE_001

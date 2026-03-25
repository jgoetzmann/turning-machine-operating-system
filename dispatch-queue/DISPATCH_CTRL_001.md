# DISPATCH_CTRL_001 — State Machine + Machine Loop

## Header
**Domain:** CTRL | **Agent:** Agent-04 | **Status:** PENDING
**Priority:** CRITICAL | **Created:** PROJECT_START
**Branch:** agent/ctrl-state-machine

## Prerequisites
- [ ] DISPATCH_TAPE_001 COMPLETE (heap needed for event queue)
- [ ] DISPATCH_DEVICE_001 COMPLETE (timer needed for `TimerTick` events)

## Context Snapshot
None — load `docs/agents/CTRL.md` only.

## Objective
Implement the Turing Machine's finite state control: `MachineState` enum, `Event` enum, `transition()` function, and `machine_loop()`. The machine loop drives ALL execution. After this task, booting transitions through `Boot → Kernel → Sleep` (no processes yet), and the loop correctly idles in `Sleep` state with `hlt`.

## Acceptance Criteria
- [ ] `MachineState` and `Event` enums match CTRL.md Section 3 exactly
- [ ] `transition(state, event) -> MachineState` handles all cases in CTRL.md table
- [ ] Unhandled (state, event) pairs → `MachineState::Panic` (never silently ignore)
- [ ] `machine_loop()` calls `transition()`, sets state, acts on new state
- [ ] `Sleep` state executes `hlt` instruction
- [ ] `Panic` state prints state dump to serial and halts forever (no return)
- [ ] `push_event(Event)` is safe to call from interrupt handlers
- [ ] `make qemu-test TEST=ctrl_states` passes (Boot→Kernel→Sleep sequence verified)
- [ ] State is never changed except via `ctrl::set_state()` (which only `transition()` calls)

## Files To Touch
- CREATE: `kernel/src/ctrl/state.rs` — MachineState, Event enums
- CREATE: `kernel/src/ctrl/transition.rs` — transition() function
- CREATE: `kernel/src/ctrl/scheduler.rs` — Phase 1 single-slot scheduler
- CREATE: `kernel/src/ctrl/mod.rs` — machine_boot(), machine_loop(), push_event()
- MODIFY: `kernel/src/head/mod.rs` — boot_init calls `ctrl::machine_boot()` at end

## Pitfalls
- See CTRL.md Section 8: `set_state()` is `pub(crate)` — enforce this
- See CTRL.md: `Panic` state must NEVER return. Use `loop { unsafe { asm!("hlt") } }`
- Event queue in interrupt context: must be safe to push from IRQ handler. Use a ring buffer with atomic head/tail (single core, so a simple spinlock is fine).

## Test
```bash
make qemu-test TEST=ctrl_states
```

## On Completion
Generate: DISPATCH_TRANS_001

---

# CTRL.md — State Machine & Scheduler Agent
> **Domain ID:** `CTRL` | **Agent Slot:** Agent-04
> **TM Role:** The finite state control unit. Decides what state the machine is in and what it does next.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

CTRL is the heart of the Turing Machine OS. It owns:

- **`MachineState` enum**: the complete set of states the machine can be in
- **`transition()` function**: the ONLY place machine state changes — maps (state, event) → new state
- **`MACHINE_STATE` global**: the current state register — readable by all, writable only by CTRL
- **Machine loop**: the top-level `loop {}` that drives all execution
- **Scheduler**: decides which process tape gets the head next (round-robin, single slot in Phase 1)
- **Event queue**: incoming events (IRQs, syscalls, timer ticks) feed the transition function

---

## 2. File Ownership

```
kernel/src/ctrl/
├── mod.rs              # machine_boot(), machine_loop(), current_state(), set_state()
├── state.rs            # MachineState enum, Event enum
├── transition.rs       # transition(state, event) -> MachineState — THE core function
└── scheduler.rs        # pick_next_process(), yield_to_kernel()
```

---

## 3. The State Machine (Complete Spec)

### States

```rust
// ctrl/state.rs
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MachineState {
    Boot      = 0x00,  // Tape being set up. No processes yet.
    Kernel    = 0x01,  // Kernel loop running. Dispatching events.
    User      = 0x02,  // A process has the head. Running on process tape.
    Interrupt = 0x03,  // IRQ received. Head is in interrupt handler.
    Sleep     = 0x04,  // No processes. Waiting for tape input (hlt).
    Halt      = 0xFE,  // Accept state. Clean shutdown.
    Panic     = 0xFF,  // Reject state. Print tape dump, halt forever.
}
```

### Events

```rust
#[derive(Copy, Clone, Debug)]
pub enum Event {
    TapeReady,               // Tape initialized, boot complete
    ProcessReady,            // A runnable process exists
    NoProcess,               // Run queue is empty
    Shutdown,                // sys_exit from init or explicit shutdown
    Syscall(u32),            // int 0x80 fired, eax = syscall number
    Irq(u8),                 // Hardware IRQ received (0-15)
    TimerTick,               // PIT tick — scheduler may preempt (Phase 2) / yield (Phase 1)
    TapeInput,               // Keyboard or serial input arrived
    Fault(FaultKind),        // CPU exception: #GP, #PF, #DF, etc.
    IrqDone,                 // IRQ handler complete, return to prior state
    ProcessExited(u32),      // Process called sys_exit(code)
}
```

### Transition Table (Complete)

```rust
// ctrl/transition.rs
pub fn transition(state: MachineState, event: Event) -> MachineState {
    use MachineState::*;
    use Event::*;
    match (state, event) {
        // Boot completes → enter kernel loop
        (Boot,      TapeReady)           => Kernel,

        // Kernel dispatches a process
        (Kernel,    ProcessReady)        => User,
        (Kernel,    NoProcess)           => Sleep,
        (Kernel,    Shutdown)            => Halt,

        // User process events
        (User,      Syscall(n))          => { handle_syscall(n); User }
        (User,      Irq(_))              => Interrupt,
        (User,      ProcessExited(_))    => Kernel,

        // Interrupt handling
        (Interrupt, IrqDone)             => User,
        (Interrupt, TimerTick)           => Kernel,  // preemption point

        // Sleep: waiting for tape input
        (Sleep,     TapeInput)           => Kernel,
        (Sleep,     Irq(_))              => Interrupt,

        // Any fault → panic
        (_,         Fault(f))            => {
            panic_handler(f);
            Panic
        }

        // Unhandled transition — always panic
        (s, e) => {
            serial_println!("CTRL: UNHANDLED TRANSITION {:?} + {:?}", s, e);
            Panic
        }
    }
}
```

---

## 4. API Contracts

```rust
// ctrl/mod.rs

/// Called by HEAD after tape is initialized. Begins the machine loop.
pub fn machine_boot() -> !;

/// The main machine loop. Runs forever until MachineState::Halt.
/// This IS the Turing Machine: read event, call transition, set state, repeat.
pub fn machine_loop() -> !;

/// Get current machine state. Safe to call from anywhere.
pub fn current_state() -> MachineState;

/// Set machine state. ONLY called from within transition(). 
/// Never call this directly from outside ctrl/.
pub(crate) fn set_state(s: MachineState);

/// Push an event into the event queue. 
/// Called by TRANS (syscalls, IRQs) to feed the machine.
pub fn push_event(event: Event);

/// Called by TRANS on timer tick. May trigger scheduler.
pub fn on_timer_tick();

// ctrl/scheduler.rs
pub fn scheduler_add(tcb: TapeControlBlock);
pub fn scheduler_remove(pid: u32);
pub fn scheduler_pick_next() -> Option<TapeControlBlock>;
pub fn scheduler_tick();       // called by timer IRQ handler
```

---

## 5. The Machine Loop (Detailed)

```rust
// ctrl/mod.rs
pub fn machine_loop() -> ! {
    loop {
        // Pull next event from the queue (or wait if empty)
        let event = match event_queue::pop() {
            Some(e) => e,
            None => {
                // No events: if in User state, continue running process
                // If in Kernel state with no process: go to Sleep
                if current_state() == MachineState::Kernel {
                    push_event(Event::NoProcess);
                }
                continue;
            }
        };

        // Run the transition function
        let next = transition(current_state(), event);
        set_state(next);

        // Act on new state
        match current_state() {
            MachineState::User      => proc::run_current_process(),
            MachineState::Sleep     => unsafe { x86::halt() },
            MachineState::Halt      => do_halt(),
            MachineState::Panic     => do_panic(),
            _                       => {}  // Kernel/Boot/Interrupt handled by transition
        }
    }
}
```

---

## 6. Scheduler (Phase 1: Cooperative)

In Phase 1, there is **one process at a time**. No preemption. The process yields by calling `sys_yield` or `sys_exit`.

```rust
// ctrl/scheduler.rs — Phase 1
static CURRENT_PROCESS: Option<TapeControlBlock> = None;

pub fn scheduler_add(tcb: TapeControlBlock) {
    CURRENT_PROCESS = Some(tcb);
}

pub fn scheduler_pick_next() -> Option<TapeControlBlock> {
    CURRENT_PROCESS.take()
}
```

Phase 2 will add a run queue with round-robin. Do NOT implement Phase 2 until Phase 1 is stable and tested.

---

## 7. Dependencies

| Depends On | What You Need |
|---|---|
| `TAPE` | Heap (for event queue), `MACHINE_STATE` storage |
| `HEAD` | `serial_println!`, `boot_init` calls `machine_boot()` |
| `TRANS` | TRANS registers IRQ/syscall handlers; CTRL calls them on events |
| `PROC` | `TapeControlBlock` type (defined in PROC, used here) |

---

## 8. Critical Constraints

- **`set_state()` is `pub(crate)` scoped to `ctrl/`**. Other domains post events, they do not set state.
- **The transition function is pure** where possible — avoid side effects inside `transition()` itself. Side effects go in the `match current_state()` block in `machine_loop()`.
- **Panic state is terminal**: once `MachineState::Panic` is set, the machine loop must call `do_panic()` and never return. `do_panic()` prints the tape dump and executes `cli; hlt`.
- **No recursion in the machine loop**: an IRQ cannot cause a new `machine_loop()` invocation. IRQ handlers post events; the loop processes them.
- **Sleep = `hlt`**: when in `MachineState::Sleep`, execute the `hlt` instruction. The CPU wakes on IRQ, which posts a `TapeInput` event.

---

## 9. Known Mistakes to Avoid

> Check `docs/MISTAKES.md` for `[CTRL]` entries.

- **MISTAKE: State changed outside transition()** — if any code sets `MACHINE_STATE` directly, state becomes inconsistent. The global is `pub(crate)` for this reason.
- **MISTAKE: Infinite loop on empty event queue without hlt** — burns CPU at 100% in `Sleep` state. Must execute `hlt`.
- **MISTAKE: Transition function has side effects with allocation** — calling `Box::new` or `Vec::push` inside `transition()` may trigger the allocator which may post an event, causing re-entrance. Keep transition() side-effect-free.

// kernel/src/ctrl/transition.rs
// The Turing Machine transition function δ(state, event) → next_state.
// Agent: CTRL (Agent-04)
//
// THIS IS THE CORE OF THE OS.
// All state changes flow through this function.
// Add new state transitions here — never change state directly.

use super::state::{Event, FaultKind, MachineState};

/// The transition function.
///
/// Given the current machine state and an incoming event,
/// returns the next machine state.
///
/// The machine loop (ctrl::machine_loop) calls this and then acts
/// on the new state. Side effects belong in machine_loop, not here.
pub fn transition(state: MachineState, event: Event) -> MachineState {
    use MachineState::*;
    use Event::*;

    match (state, event) {
        // ── Boot transitions ──────────────────────────────────────────
        (Boot, TapeReady)           => Kernel,

        // ── Kernel transitions ────────────────────────────────────────
        (Kernel, ProcessReady)      => User,
        (Kernel, NoProcess)         => Sleep,
        (Kernel, Shutdown)          => Halt,
        (Kernel, Irq(n))            => {
            crate::serial_println!("CTRL: kernel received IRQ {}", n);
            Interrupt
        }

        // ── User transitions ──────────────────────────────────────────
        (User, Syscall(_n))         => {
            // Syscall is handled inline by TRANS, then returns to User
            // The state stays User unless the syscall causes exit/fault
            User
        }
        (User, Irq(_))              => Interrupt,
        (User, ProcessExited(code)) => {
            crate::serial_println!("CTRL: process exited with code {}", code);
            Kernel
        }

        // ── Interrupt transitions ─────────────────────────────────────
        (Interrupt, IrqDone)        => User,     // return to interrupted process
        (Interrupt, TimerTick)      => Kernel,   // timer tick → scheduler runs

        // ── Sleep transitions ─────────────────────────────────────────
        (Sleep, TapeInput)          => Kernel,
        (Sleep, Irq(_))             => Interrupt,
        (Sleep, TimerTick)          => {
            // Timer in sleep — check if a process became ready
            Kernel
        }

        // ── Fault: always panic ───────────────────────────────────────
        (_, Fault(f)) => {
            crate::serial_println!("CTRL: FAULT → Panic: {:?}", f);
            Panic
        }

        // ── Unhandled transition — always panic ───────────────────────
        (s, e) => {
            crate::serial_println!(
                "CTRL: UNHANDLED TRANSITION ({:?}, {:?}) → Panic",
                s, e
            );
            Panic
        }
    }
}

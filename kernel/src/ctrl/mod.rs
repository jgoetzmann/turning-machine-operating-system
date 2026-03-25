// kernel/src/ctrl/mod.rs
// CTRL domain — Finite State Control Unit (the TM state machine)
// Agent: Agent-04
// Read docs/agents/CTRL.md before modifying.

pub mod state;       // MachineState, Event enums
pub mod transition;  // transition(state, event) -> MachineState
pub mod scheduler;   // pick_next_process, Phase 1 single-slot

use state::{MachineState, Event};

// The current machine state — only mutated by set_state()
static mut MACHINE_STATE: MachineState = MachineState::Boot;

/// Get the current TM state. Safe to call from any domain.
pub fn current_state() -> MachineState {
    // SAFETY: Single core. No races. Reads are atomic on x86-32.
    unsafe { MACHINE_STATE }
}

/// Set the machine state. ONLY called from within transition().
/// pub(crate) — other domains must use push_event() instead.
pub(crate) fn set_state(s: MachineState) {
    // SAFETY: Single core. Called only from transition() which is called
    //         from machine_loop() in a single-threaded dispatch.
    unsafe { MACHINE_STATE = s; }
}

/// Push an event into the machine's event queue.
/// Safe to call from interrupt handlers.
pub fn push_event(event: Event) {
    // TODO (Agent-04): implement event queue (ring buffer)
    // For now, process immediately (works for single-core, non-reentrant Phase 1)
    let _ = event;
}

/// Called by HEAD after tape and devices are initialized.
/// Registers all interrupt/syscall handlers, then starts the machine loop.
pub fn machine_boot() -> ! {
    crate::serial_println!("CTRL: machine boot — registering handlers");

    // Initialize devices
    crate::device::init_all_devices();

    // Initialize tape
    crate::tape::init_tape(crate::head::get_tape_memory_map());

    // Register all interrupt/syscall gates (TRANS domain)
    // TODO (Agent-04/05 coordination): crate::trans::register_all_handlers(&mut idt);

    // Enable interrupts
    unsafe { core::arch::asm!("sti") };
    crate::serial_println!("CTRL: interrupts enabled");

    // Transition: Boot → Kernel
    let next = transition::transition(current_state(), Event::TapeReady);
    set_state(next);
    crate::serial_println!("CTRL: state → {:?}", current_state());

    // Enter the TM main loop
    machine_loop()
}

/// The Turing Machine main loop.
/// This drives ALL execution. Runs until MachineState::Halt.
pub fn machine_loop() -> ! {
    loop {
        match current_state() {
            MachineState::Sleep => {
                // Waiting for tape input — execute hlt to save power
                // CPU will wake on next interrupt
                unsafe { core::arch::asm!("hlt") };
            }
            MachineState::Halt => {
                crate::serial_println!("CTRL: machine halted (accept state)");
                loop { unsafe { core::arch::asm!("cli; hlt") }; }
            }
            MachineState::Panic => {
                crate::serial_println!("CTRL: machine panic (reject state)");
                loop { unsafe { core::arch::asm!("cli; hlt") }; }
            }
            MachineState::Kernel => {
                // TODO (Agent-04): check run queue, dispatch processes
                // For now, transition to Sleep if no process ready
                let next = transition::transition(current_state(), Event::NoProcess);
                set_state(next);
            }
            MachineState::User => {
                // TODO (Agent-04): resume running the current process
                // The process runs until it yields, syscalls, or is interrupted
            }
            _ => {}
        }
    }
}

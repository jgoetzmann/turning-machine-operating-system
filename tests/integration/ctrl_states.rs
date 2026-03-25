// tests/integration/ctrl_states.rs
// Test: state machine transitions fire correctly.
// Domain: CTRL | Run: make qemu-test TEST=ctrl_states

pub fn run() {
    use crate::ctrl::{state::{MachineState, Event}, transition::transition};
    crate::serial_println!("[TEST START: ctrl_states]");

    // Boot → Kernel on TapeReady
    let s = transition(MachineState::Boot, Event::TapeReady);
    assert_eq!(s, MachineState::Kernel, "Boot+TapeReady should → Kernel");

    // Kernel → Sleep on NoProcess
    let s = transition(MachineState::Kernel, Event::NoProcess);
    assert_eq!(s, MachineState::Sleep, "Kernel+NoProcess should → Sleep");

    // Kernel → Halt on Shutdown
    let s = transition(MachineState::Kernel, Event::Shutdown);
    assert_eq!(s, MachineState::Halt, "Kernel+Shutdown should → Halt");

    // Kernel → User on ProcessReady
    let s = transition(MachineState::Kernel, Event::ProcessReady);
    assert_eq!(s, MachineState::User, "Kernel+ProcessReady should → User");

    // User → Interrupt on Irq
    let s = transition(MachineState::User, Event::Irq(1));
    assert_eq!(s, MachineState::Interrupt, "User+Irq should → Interrupt");

    // Interrupt → User on IrqDone
    let s = transition(MachineState::Interrupt, Event::IrqDone);
    assert_eq!(s, MachineState::User, "Interrupt+IrqDone should → User");

    // Any state + Fault → Panic
    let s = transition(MachineState::User, Event::Fault(
        crate::ctrl::state::FaultKind::DivideError
    ));
    assert_eq!(s, MachineState::Panic, "Any+Fault should → Panic");

    // Sleep → Kernel on TapeInput
    let s = transition(MachineState::Sleep, Event::TapeInput);
    assert_eq!(s, MachineState::Kernel, "Sleep+TapeInput should → Kernel");

    crate::serial_println!("  CTRL: all {} transitions verified", 8);
    crate::serial_println!("[TEST PASS: ctrl_states]");
    super::head_boot_basic::qemu_exit_success();
}

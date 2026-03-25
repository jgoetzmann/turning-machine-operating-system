// kernel/src/ctrl/state.rs
// MachineState and Event enums — the TM state alphabet.
// Agent: CTRL (Agent-04)
// See MASTER.md Section 4 for state semantics.

/// The finite set of states the Turing Machine OS can be in.
/// Stored in the global MACHINE_STATE register.
///
/// State transitions are ONLY made via ctrl::transition().
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MachineState {
    Boot      = 0x00,  // Initializing tape regions, setting up head
    Kernel    = 0x01,  // Kernel main loop — reading tape, dispatching events
    User      = 0x02,  // A process has the head; executing on process tape
    Interrupt = 0x03,  // Head interrupted — saving position, handling IRQ
    Sleep     = 0x04,  // No runnable process; waiting for tape input (hlt)
    Halt      = 0xFE,  // Accept state — clean shutdown
    Panic     = 0xFF,  // Reject state — unrecoverable error
}

/// Kinds of CPU exceptions that transition to Panic state.
#[derive(Copy, Clone, Debug)]
pub enum FaultKind {
    DivideError,         // #DE vector 0
    InvalidOpcode,       // #UD vector 6
    DoubleFault,         // #DF vector 8 — always fatal
    GeneralProtection(u32),  // #GP vector 13, error code
    PageFault(u32),      // #PF vector 14 — should never fire (paging disabled)
    Other(u8),           // any other exception vector
}

/// Events that drive the TM state transition function.
/// Posted by interrupt handlers, syscall dispatch, and the machine loop itself.
#[derive(Copy, Clone, Debug)]
pub enum Event {
    TapeReady,               // Boot complete, tape initialized
    ProcessReady,            // A runnable process is in the scheduler
    NoProcess,               // Run queue is empty
    Shutdown,                // sys_exit(0) from init process
    Syscall(u32),            // int 0x80: eax = syscall number
    Irq(u8),                 // Hardware IRQ 0-15 fired
    TimerTick,               // PIT IRQ0 — 100Hz scheduler tick
    TapeInput,               // Keyboard or serial data arrived
    IrqDone,                 // IRQ handler complete
    ProcessExited(u32),      // Process called sys_exit(code)
    Fault(FaultKind),        // CPU exception — always → Panic
}

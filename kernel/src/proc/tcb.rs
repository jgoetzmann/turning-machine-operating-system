// kernel/src/proc/tcb.rs
// Tape Control Block — the saved head state of a process.
// Agent: PROC (Agent-09)
//
// In TM terms: a process is a sub-tape with a saved head position.
// Switching processes = saving the head's internal register state,
// then loading a different saved state.
//
// Must be #[repr(C)] because context.rs reads it from assembly.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcState {
    Ready   = 0,
    Running = 1,
    Blocked = 2,
    Zombie  = 3,
}

/// TapeControlBlock — complete saved state of the tape head (CPU) for one process.
///
/// Layout is fixed by context.rs assembly. Do not reorder fields.
/// All fields are u32 (32-bit — this is i686 architecture).
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TapeControlBlock {
    // Saved CPU registers — the head's internal state
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,       // stack pointer (tape head position on stack)
    pub eip: u32,       // instruction pointer (tape head read position)
    pub eflags: u32,    // CPU flags (must have IF=1 for interrupts to work)

    // Process metadata
    pub pid:       u32,
    pub state:     ProcState,
    pub tape_base: u32,         // start of this process's tape region
    pub tape_end:  u32,         // end of this process's tape region
    pub exit_code: u32,         // set by sys_exit

    // Process name (for debug output)
    pub name: [u8; 8],
}

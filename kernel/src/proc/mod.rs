// kernel/src/proc/mod.rs
// PROC domain — Process model (sub-tape + saved head state)
// Agent: Agent-09
// Read docs/agents/PROC.md before modifying.

pub mod tcb;      // TapeControlBlock — saved CPU state per process
pub mod context;  // context_switch — save/restore head position

pub use tcb::TapeControlBlock;

use crate::tape::regions::{PROC_TAPE_BASE, PROC_TAPE_END, PROC_STACK_TOP};

#[derive(Debug)]
pub enum ProcError {
    NoSlotAvailable,
    InvalidEntry,
    TapeRegionFull,
}

/// Create a new process with entry point `entry` in the process tape region.
///
/// `entry` is a physical address within `PROC_TAPE_BASE..PROC_TAPE_END`.
/// The process will start executing at `entry` with `esp = PROC_STACK_TOP`.
///
/// Returns the PID (always 1 in Phase 1).
pub fn create_process(entry: u32, tape_base: u32, _tape_size: u32) -> Result<u32, ProcError> {
    if entry < PROC_TAPE_BASE || entry >= PROC_TAPE_END {
        return Err(ProcError::InvalidEntry);
    }

    let tcb = TapeControlBlock {
        eax: 0, ebx: 0, ecx: 0, edx: 0,
        esi: 0, edi: 0, ebp: 0,
        esp: PROC_STACK_TOP - 4,  // leave room for return address
        eip: entry,
        // EFLAGS: bit 9 (IF) set so process runs with interrupts enabled
        eflags: 0x0200,
        pid: 1,  // Phase 1: always PID 1
        state: tcb::ProcState::Ready,
        tape_base,
        tape_end: PROC_TAPE_END,
        exit_code: 0,
        name: *b"init\0\0\0\0",
    };

    crate::ctrl::scheduler::scheduler_add(tcb);
    crate::serial_println!("PROC: created process PID=1 entry={:#010x}", entry);
    Ok(1)
}

/// Destroy a process and free its resources.
pub fn destroy_process(pid: u32) {
    crate::ctrl::scheduler::scheduler_remove(pid);
    crate::serial_println!("PROC: destroyed process PID={}", pid);
}

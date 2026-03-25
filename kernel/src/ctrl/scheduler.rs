// kernel/src/ctrl/scheduler.rs
// Process scheduler — Phase 1: single-slot cooperative.
// Agent: CTRL (Agent-04)
//
// Phase 1: one process at a time. No preemption. No run queue.
// The "scheduler" is just a single TCB slot that the kernel loads and runs.
// The process yields by calling sys_yield or sys_exit.
//
// Phase 2 (do NOT implement yet): round-robin run queue with timer preemption.

use crate::proc::TapeControlBlock;

/// The single process slot. Phase 1 only.
static mut CURRENT: Option<TapeControlBlock> = None;

/// Add a process to the scheduler.
/// Phase 1: replaces any existing process (only one slot).
pub fn scheduler_add(tcb: TapeControlBlock) {
    // SAFETY: Single core. Called from kernel context with interrupts disabled.
    unsafe { CURRENT = Some(tcb); }
    crate::serial_println!("CTRL: scheduler: process {} added (pid={})", 
        core::str::from_utf8(&tcb.name[..4]).unwrap_or("?"),
        tcb.pid);
}

/// Remove a process from the scheduler by PID.
pub fn scheduler_remove(pid: u32) {
    // SAFETY: Single core.
    unsafe {
        if let Some(ref tcb) = CURRENT {
            if tcb.pid == pid {
                CURRENT = None;
            }
        }
    }
}

/// Pick the next process to run. Returns None if run queue is empty.
/// Removes the process from the "queue" (single slot — it runs to completion or yield).
pub fn scheduler_pick_next() -> Option<TapeControlBlock> {
    // SAFETY: Single core.
    unsafe { CURRENT.take() }
}

/// Called on every timer tick. Phase 1: no preemption, just notes the tick.
pub fn scheduler_tick() {
    // Phase 2: this is where preemption logic goes.
    // Phase 1: do nothing — process runs until it yields voluntarily.
}

/// Returns true if there is a runnable process.
pub fn scheduler_has_process() -> bool {
    // SAFETY: Single core.
    unsafe { CURRENT.is_some() }
}

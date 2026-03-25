// tests/integration/proc_create.rs
// Test: create a process TCB and verify its initial state.
// Domain: PROC | Run: make qemu-test TEST=proc_create

pub fn run() {
    use crate::tape::regions::{PROC_TAPE_BASE, PROC_STACK_TOP};
    crate::serial_println!("[TEST START: proc_create]");

    // Create a process with a dummy entry point in the process tape region
    let entry = PROC_TAPE_BASE + 0x1000;  // arbitrary entry in valid range
    let pid = match crate::proc::create_process(entry, PROC_TAPE_BASE, 0x1000) {
        Ok(pid) => pid,
        Err(e) => {
            crate::serial_println!("[TEST FAIL: proc_create] create failed: {:?}", e);
            super::head_boot_basic::qemu_exit_failure("create_process failed");
        }
    };

    assert_eq!(pid, 1, "Phase 1 PID must be 1");

    // Verify TCB in scheduler
    assert!(crate::ctrl::scheduler::scheduler_has_process(), "process not in scheduler");

    // Peek at the TCB
    let tcb = crate::ctrl::scheduler::scheduler_pick_next()
        .expect("scheduler_pick_next returned None after add");

    assert_eq!(tcb.eip, entry, "TCB.eip != entry");
    assert_eq!(tcb.esp, PROC_STACK_TOP - 4, "TCB.esp != stack top - 4");
    assert_eq!(tcb.eflags & 0x200, 0x200, "TCB.eflags IF bit not set");
    assert_eq!(tcb.pid, 1, "TCB.pid != 1");

    crate::serial_println!("  PROC: TCB eip={:#010x} esp={:#010x} eflags={:#010x}",
        tcb.eip, tcb.esp, tcb.eflags);
    crate::serial_println!("[TEST PASS: proc_create]");
    super::head_boot_basic::qemu_exit_success();
}

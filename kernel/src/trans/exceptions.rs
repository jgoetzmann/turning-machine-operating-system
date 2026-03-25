// kernel/src/trans/exceptions.rs
// CPU exception handlers — all route to MachineState::Panic.
// Agent: TRANS (Agent-05)

use super::irq::InterruptFrame;
use crate::ctrl::state::{Event, FaultKind};

/// #DE — Divide Error (vector 0x00)
pub extern "x86-interrupt" fn divide_error_handler(frame: &InterruptFrame) {
    crate::serial_println!("TRANS: #DE Divide Error at EIP={:#010x}", frame.eip);
    crate::ctrl::push_event(Event::Fault(FaultKind::DivideError));
    loop { unsafe { core::arch::asm!("hlt") }; }
}

/// #UD — Invalid Opcode (vector 0x06)
pub extern "x86-interrupt" fn invalid_opcode_handler(frame: &InterruptFrame) {
    crate::serial_println!("TRANS: #UD Invalid Opcode at EIP={:#010x}", frame.eip);
    crate::ctrl::push_event(Event::Fault(FaultKind::InvalidOpcode));
    loop { unsafe { core::arch::asm!("hlt") }; }
}

/// #DF — Double Fault (vector 0x08) — always fatal, has error code (always 0)
/// The CPU pushes an error code for #DF (always 0).
pub extern "x86-interrupt" fn double_fault_handler(frame: &InterruptFrame, _error: u32) -> ! {
    crate::serial_println!("TRANS: #DF DOUBLE FAULT at EIP={:#010x} — HALTING", frame.eip);
    crate::serial_println!("TRANS: This usually means stack overflow or corrupt IDT gate.");
    crate::ctrl::push_event(Event::Fault(FaultKind::DoubleFault));
    loop { unsafe { core::arch::asm!("cli; hlt") }; }
}

/// #GP — General Protection Fault (vector 0x0D) — has error code
pub extern "x86-interrupt" fn general_protection_handler(frame: &InterruptFrame, error: u32) {
    crate::serial_println!("TRANS: #GP General Protection at EIP={:#010x} error={:#010x}",
        frame.eip, error);
    crate::serial_println!("TRANS: Common causes:");
    crate::serial_println!("  - Syscall gate DPL=0 (fix: DPL=3 for int 0x80)");
    crate::serial_println!("  - Invalid segment selector");
    crate::serial_println!("  - Privileged instruction from ring 3");
    crate::ctrl::push_event(Event::Fault(FaultKind::GeneralProtection(error)));
    loop { unsafe { core::arch::asm!("cli; hlt") }; }
}

/// #PF — Page Fault (vector 0x0E) — has error code, CR2 = faulting address
///
/// SHOULD NEVER FIRE — paging is disabled on this OS (INV-04).
/// If it does fire, something is very wrong.
pub extern "x86-interrupt" fn page_fault_handler(frame: &InterruptFrame, error: u32) {
    // CR2 contains the faulting virtual address
    let cr2: u32;
    unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2); }
    crate::serial_println!("TRANS: #PF PAGE FAULT — THIS SHOULD NOT HAPPEN (paging is disabled!)");
    crate::serial_println!("  EIP={:#010x}  CR2={:#010x}  error={:#010x}", frame.eip, cr2, error);
    crate::serial_println!("  Did someone accidentally enable CR0.PG? See INV-04.");
    crate::ctrl::push_event(Event::Fault(FaultKind::PageFault(cr2)));
    loop { unsafe { core::arch::asm!("cli; hlt") }; }
}

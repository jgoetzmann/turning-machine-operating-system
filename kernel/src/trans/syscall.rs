// kernel/src/trans/syscall.rs
// int 0x80 syscall entry and dispatch table.
// Agent: TRANS (Agent-05)
//
// ABI: 32-bit Linux i386 compatible (int 0x80)
//   eax = syscall number
//   ebx = arg1, ecx = arg2, edx = arg3
//   return value in eax (negative = -errno)
//
// CRITICAL: The entry point is a #[naked] function.
// The compiler adds NO prologue or epilogue.
// We must manually save/restore ALL registers.
//
// See MISTAKE-004: syscall gate MUST be DPL=3.
// See docs/agents/TRANS.md for full ABI documentation.

/// Syscall numbers — Linux i386 ABI compatible.
/// Using the same numbers means future compat with Linux binaries.
pub mod nr {
    pub const EXIT:    u32 = 1;
    pub const FORK:    u32 = 2;   // Phase 2
    pub const READ:    u32 = 3;
    pub const WRITE:   u32 = 4;
    pub const OPEN:    u32 = 5;
    pub const CLOSE:   u32 = 6;
    pub const GETPID:  u32 = 20;
    pub const BRK:     u32 = 45;
    pub const YIELD:   u32 = 158; // sched_yield
}

/// Error codes (negative return values)
pub mod errno {
    pub const ENOSYS:  u32 = 38;  // Function not implemented
    pub const EBADF:   u32 = 9;   // Bad file descriptor
    pub const EINVAL:  u32 = 22;  // Invalid argument
    pub const EFAULT:  u32 = 14;  // Bad address
}

/// The syscall entry point — invoked by `int 0x80` from userspace.
///
/// This is a naked function: the compiler generates no prologue/epilogue.
/// We manually save all registers, call the Rust dispatcher, then restore.
///
/// Stack layout on entry (from ring 3 → ring 0 via TSS):
///   esp+0:  eip  (user instruction pointer, saved by CPU)
///   esp+4:  cs   (user code segment, 0x1B)
///   esp+8:  eflags
///   esp+12: esp  (user stack pointer, only if privilege change)
///   esp+16: ss   (user stack segment, only if privilege change)
///
/// # Safety
/// Called by CPU on int 0x80. Must not be called directly.
#[naked]
pub unsafe extern "C" fn syscall_entry() {
    // SAFETY: This is a naked ISR entry. We own the stack completely.
    // pushad saves: eax ecx edx ebx esp ebp esi edi (in that order, 32 bytes)
    core::arch::asm!(
        // Save all general-purpose registers
        "pushad",

        // The pushad above saved eax (syscall number) at [esp+28].
        // syscall args: eax=num, ebx=arg1, ecx=arg2, edx=arg3
        // Retrieve them from the saved register frame.
        // esp points to the pushad frame: edi esi ebp esp_old ebx edx ecx eax
        //   [esp+0]  = edi
        //   [esp+4]  = esi
        //   [esp+8]  = ebp
        //   [esp+12] = esp (old, ignored)
        //   [esp+16] = ebx  ← arg1
        //   [esp+20] = edx  ← arg3
        //   [esp+24] = ecx  ← arg2
        //   [esp+28] = eax  ← syscall number

        // Set up segment registers for kernel data
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",

        // Call Rust dispatcher: syscall_dispatch(num, arg1, arg2, arg3)
        // Arguments in registers (fastcall would be cleaner but cdecl works):
        "push dword ptr [esp+20]",   // arg3 = edx (saved at offset 20)
        "push dword ptr [esp+28]",   // arg2 = ecx (saved at offset 24+4 after push)
        "push dword ptr [esp+36]",   // arg1 = ebx (saved at offset 16+8+12 after 2 pushes) 
        "push dword ptr [esp+44]",   // num  = eax (saved at offset 28+16)
        "call {dispatch}",
        "add esp, 16",               // clean up 4 args from stack

        // Store return value back into saved eax slot so it's restored by popad
        "mov dword ptr [esp+28], eax",

        // Restore all registers and return to userspace
        "popad",
        "iretd",                     // 32-bit interrupt return

        dispatch = sym syscall_dispatch,
        options(noreturn)
    );
}

/// Syscall dispatcher — called from syscall_entry with interrupts disabled.
/// Returns the value to place in userspace eax.
#[no_mangle]
pub extern "C" fn syscall_dispatch(num: u32, arg1: u32, arg2: u32, arg3: u32) -> u32 {
    match num {
        nr::EXIT => {
            crate::serial_println!("TRANS: sys_exit({})", arg1);
            crate::ctrl::push_event(crate::ctrl::state::Event::ProcessExited(arg1));
            0
        }

        nr::WRITE => {
            sys_write(arg1, arg2, arg3)
        }

        nr::GETPID => {
            // TODO (Agent-05): get PID from current process TCB
            1  // placeholder: always PID 1
        }

        nr::YIELD => {
            // Voluntarily yield — post ProcessExited so kernel reschedules
            // TODO (Agent-05): proper yield (put process back in scheduler, not exit)
            crate::ctrl::push_event(crate::ctrl::state::Event::Irq(0));
            0
        }

        _ => {
            crate::serial_println!("TRANS: unimplemented syscall {}", num);
            0u32.wrapping_sub(errno::ENOSYS)  // -ENOSYS
        }
    }
}

/// sys_write(fd, buf_addr, len) — write bytes to a file descriptor.
///
/// fd=1 (stdout) and fd=2 (stderr) → VGA + serial output.
fn sys_write(fd: u32, buf_addr: u32, len: u32) -> u32 {
    // Security check: buf_addr must be in userspace (below kernel)
    if buf_addr >= crate::tape::regions::KERNEL_CODE_BASE {
        return 0u32.wrapping_sub(errno::EFAULT);
    }
    if buf_addr.checked_add(len).is_none() {
        return 0u32.wrapping_sub(errno::EINVAL);
    }

    match fd {
        1 | 2 => {
            // stdout / stderr → write to VGA + serial
            // SAFETY: We validated buf_addr is in userspace range above.
            //         The process tape region is mapped and accessible (flat memory).
            let bytes = unsafe {
                core::slice::from_raw_parts(buf_addr as *const u8, len as usize)
            };
            if let Ok(s) = core::str::from_utf8(bytes) {
                // Write to display
                crate::display::kprint_str(s);
                // Also to serial for test capture
                crate::head::serial::serial_write_str(s);
            }
            len  // return bytes written
        }
        _ => 0u32.wrapping_sub(errno::EBADF),
    }
}

// kernel/src/main.rs
// Turing Machine OS — Kernel entry point
//
// This file is the crate root. It declares all modules and provides
// the panic handler and OOM handler required by no_std.
//
// The actual _start symbol is in head/boot.asm (NASM assembly).
// boot.asm calls boot_init() defined in head/mod.rs.
// boot_init() calls ctrl::machine_boot() which runs the TM loop forever.
//
// DO NOT add logic here — route everything through the domain modules.

// ── Required feature flags for bare-metal kernel ──────────────────────────
#![no_std]                          // No standard library — ever
#![no_main]                         // Custom entry point (_start in boot.asm)
#![feature(abi_x86_interrupt)]      // extern "x86-interrupt" for IDT handlers
#![feature(alloc_error_handler)]    // Custom OOM handler
#![feature(naked_functions)]        // #[naked] for context switch + syscall entry

// ── Allow the global allocator (needed for Box, Vec, etc.) ────────────────
extern crate alloc;

// ── Domain modules ────────────────────────────────────────────────────────
// Each module is owned by one agent. See MASTER.md Section 6.

pub mod head;       // HEAD   (Agent-02): boot, GDT, IDT skeleton, serial
pub mod tape;       // TAPE   (Agent-03): memory regions, allocator, heap
pub mod ctrl;       // CTRL   (Agent-04): state machine, machine loop, scheduler
pub mod trans;      // TRANS  (Agent-05): syscall dispatch, IRQ handlers
pub mod device;     // DEVICE (Agent-06): PIC, PIT, keyboard, ATA
pub mod display;    // DISPLAY(Agent-07): VGA text mode writer
pub mod store;      // STORE  (Agent-08): FAT12 filesystem
pub mod proc;       // PROC   (Agent-09): TCB, context switch

// ── Test infrastructure (integration tests compiled into kernel) ───────────
// Tests run when kernel is launched with `-append "test=test_name"`.
pub mod test_dispatch;
pub mod tests {
    pub mod head_boot_basic;
    pub mod tape_alloc;
    pub mod ctrl_states;
    pub mod display_write;
    pub mod store_fat12;
    pub mod proc_create;
    pub mod device_timer;
}

// ── Panic handler — REQUIRED for no_std ───────────────────────────────────
//
// Called on panic!(), assert!(), index out of bounds, etc.
// Transitions the TM to MachineState::Panic.
// Must never return.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Try to print panic info — use serial (always works) + VGA if available
    //
    // SAFETY: We're in a panic — it's acceptable to force-unlock mutexes
    // that may be poisoned. We cannot safely acquire them.
    head::serial::serial_write_str("\n\n!!! KERNEL PANIC !!!\n");
    if let Some(location) = info.location() {
        head::serial::serial_write_str(location.file());
        head::serial::serial_write_str(":");
        // Format line number without alloc
        let line = location.line();
        let mut buf = [0u8; 10];
        let s = u32_to_str(line, &mut buf);
        head::serial::serial_write_str(s);
        head::serial::serial_write_str("\n");
    }
    if let Some(msg) = info.message().as_str() {
        head::serial::serial_write_str(msg);
        head::serial::serial_write_str("\n");
    }

    // Transition to Panic state (if CTRL is initialized)
    // If not, just halt — we're early in boot
    loop {
        unsafe { core::arch::asm!("cli; hlt") };
    }
}

// ── OOM handler — REQUIRED for alloc ─────────────────────────────────────
#[alloc_error_handler]
fn oom(_layout: core::alloc::Layout) -> ! {
    head::serial::serial_write_str("\n!!! KERNEL OOM: heap exhausted !!!\n");
    loop {
        unsafe { core::arch::asm!("cli; hlt") };
    }
}

// ── Utility: format u32 into a stack buffer without alloc ─────────────────
fn u32_to_str(mut n: u32, buf: &mut [u8; 10]) -> &str {
    let mut i = buf.len();
    if n == 0 {
        buf[i - 1] = b'0';
        return core::str::from_utf8(&buf[i - 1..]).unwrap_or("?");
    }
    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    core::str::from_utf8(&buf[i..]).unwrap_or("?")
}

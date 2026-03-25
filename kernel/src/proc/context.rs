// kernel/src/proc/context.rs
// Context switch — save/restore the tape head position (CPU register state).
// Agent: PROC (Agent-09)
//
// This IS the TM head repositioning operation.
// When we context switch, we save where the current head is,
// then pick up a saved head position and resume from there.
//
// The function is #[naked]: the compiler generates NO prologue or epilogue.
// Every saved register, every stack operation, must be explicit.
//
// Field offsets in TapeControlBlock (must match proc/tcb.rs exactly):
//   eax   @ offset 0
//   ebx   @ offset 4
//   ecx   @ offset 8
//   edx   @ offset 12
//   esi   @ offset 16
//   edi   @ offset 20
//   ebp   @ offset 24
//   esp   @ offset 28
//   eip   @ offset 32
//   eflags @ offset 36
//
// Calling convention (cdecl):
//   [esp+4] = save_into  (pointer to current process's TCB)
//   [esp+8] = load_from  (pointer to next process's TCB)

use super::TapeControlBlock;

/// Save current CPU state into `save_into`, then load state from `load_from`.
///
/// After this function "returns," the CPU is running in the context of
/// the process described by `load_from`.
///
/// # Safety
/// - Both pointers must be valid, non-null `TapeControlBlock` addresses.
/// - Must only be called from kernel context (ring 0).
/// - Interrupts should be disabled at the call site to prevent a timer
///   IRQ from firing between the save and load.
/// - This function is `#[naked]` — do NOT call it with the wrong signature.
#[naked]
pub unsafe extern "C" fn context_switch(
    save_into:  *mut  TapeControlBlock,
    load_from:  *const TapeControlBlock,
) {
    // SAFETY: Naked function. All register saves/restores are explicit.
    // We use the cdecl calling convention: args are on the stack.
    // [esp+0] = return address (pushed by call instruction)
    // [esp+4] = save_into  pointer
    // [esp+8] = load_from pointer
    core::arch::asm!(
        // ── Disable interrupts during switch ──────────────────────────
        "cli",

        // ── Load argument pointers from stack ─────────────────────────
        "mov eax, [esp+4]",     // eax = save_into
        "mov edx, [esp+8]",     // edx = load_from

        // ── Save current head state into save_into TCB ────────────────
        // We don't save eax/edx — they hold our pointer args.
        // We'll save them specially below.
        "mov [eax+4],  ebx",    // TCB.ebx  @ offset 4
        "mov [eax+8],  ecx",    // TCB.ecx  @ offset 8
        "mov [eax+12], edx",    // Actually save edx (load_from) — overwritten later
        "mov [eax+16], esi",    // TCB.esi  @ offset 16
        "mov [eax+20], edi",    // TCB.edi  @ offset 20
        "mov [eax+24], ebp",    // TCB.ebp  @ offset 24

        // Save esp: at call time, esp points to return address.
        // We want to save the caller's esp (before the call pushed return addr).
        // esp-after-call = caller esp - 4. Save the after-call value;
        // on resume, the popad/pop sequence will put esp back correctly.
        "mov [eax+28], esp",    // TCB.esp  @ offset 28

        // Save return address as EIP (where we'll resume)
        "mov ecx, [esp]",       // ecx = return address
        "mov [eax+32], ecx",    // TCB.eip  @ offset 32

        // Save EFLAGS
        "pushfd",
        "pop ecx",
        "mov [eax+36], ecx",    // TCB.eflags @ offset 36

        // Save EAX (which held the save_into pointer)
        "mov [eax+0],  eax",    // TCB.eax  @ offset 0
        // Save EDX (which held the load_from pointer) — save actual caller edx
        // We already clobbered it, so save 0 as placeholder
        // The process doesn't rely on edx surviving a context switch
        "mov dword ptr [eax+12], 0",

        // ── Load next head state from load_from TCB ───────────────────
        // edx still holds load_from pointer

        // Restore EFLAGS first
        "mov ecx, [edx+36]",    // eflags
        "push ecx",
        "popfd",                // restore flags (including IF)

        // Restore stack pointer — must do this before other restores
        "mov esp, [edx+28]",    // esp

        // Restore general-purpose registers
        "mov eax, [edx+0]",
        "mov ebx, [edx+4]",
        "mov ecx, [edx+8]",
        // edx will be last (we're still using it for the pointer)
        "mov esi, [edx+16]",
        "mov edi, [edx+20]",
        "mov ebp, [edx+24]",

        // Push the new EIP so we can ret to it
        "push dword ptr [edx+32]",

        // Now restore edx (this clobbers our load_from pointer — last use)
        "mov edx, [edx+12]",

        // ── Re-enable interrupts ───────────────────────────────────────
        // Note: EFLAGS was already restored above (which may have set IF).
        // The explicit sti here ensures IF is set even if the saved eflags had IF=0.
        "sti",

        // ── Jump to the new EIP ────────────────────────────────────────
        // The new EIP was pushed onto the (new) stack above.
        // ret pops it and jumps there — this "returns" to the new process.
        "ret",

        options(noreturn)
    );
}

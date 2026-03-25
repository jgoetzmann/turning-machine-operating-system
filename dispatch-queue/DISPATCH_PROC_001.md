# DISPATCH_PROC_001 — Tape Control Block + Context Switch

## Header
**Domain:** PROC | **Agent:** Agent-09 | **Status:** PENDING
**Priority:** HIGH | **Created:** PROJECT_START
**Branch:** agent/proc-tcb-context-switch

## Prerequisites
- [ ] DISPATCH_TAPE_001 COMPLETE — heap for TCB allocation
- [ ] DISPATCH_CTRL_001 COMPLETE — scheduler calls context_switch

## Context Snapshot
None — load `docs/agents/PROC.md` only.

## Objective
Implement the `TapeControlBlock` (saved head state) and 32-bit context switch. When `context_switch(save_into, load_from)` is called, the current CPU register state is saved into `save_into` and the state from `load_from` is restored. After this task, the kernel can create and run a process from a known entry point.

## Acceptance Criteria
- [ ] `TapeControlBlock` struct matches PROC.md Section 3 exactly, `#[repr(C)]`
- [ ] `context_switch(save: *mut TCB, load: *const TCB)` is a `#[naked]` 32-bit function
- [ ] Context switch saves: `eax ebx ecx edx esi edi ebp esp eip eflags`
- [ ] Context switch restores all the above and `jmp` to restored `eip`
- [ ] `create_process(entry: u32, tape_base: u32, tape_size: u32) -> u32` returns PID
- [ ] New process `esp` set to `tape::regions::PROC_STACK_TOP`
- [ ] New process `eip` set to `entry` parameter
- [ ] New process tape region is within `PROC_TAPE_BASE..PROC_TAPE_END`
- [ ] `make qemu-test TEST=proc_create` passes (create process, run entry, sys_exit)
- [ ] `make qemu-test TEST=proc_context_switch` passes (save + restore registers verified)
- [ ] **Context switch is 32-bit: `pushal`/`popal` or explicit `eax..ebp` push/pop**

## Files To Touch
- CREATE: `kernel/src/proc/tcb.rs` — TapeControlBlock, ProcState
- CREATE: `kernel/src/proc/context.rs` — context_switch (naked asm)
- CREATE: `kernel/src/proc/mod.rs` — create_process, destroy_process, get_tcb
- CREATE: `tests/integration/proc_create.rs`
- CREATE: `tests/integration/proc_context_switch.rs`

## API Spec
```rust
#[repr(C)]
pub struct TapeControlBlock {
    pub eax: u32, pub ebx: u32, pub ecx: u32, pub edx: u32,
    pub esi: u32, pub edi: u32, pub ebp: u32, pub esp: u32,
    pub eip: u32, pub eflags: u32,
    pub pid: u32, pub state: ProcState,
    pub tape_base: u32, pub tape_end: u32, pub exit_code: u32,
}
#[naked]
pub unsafe extern "C" fn context_switch(
    save_into: *mut TapeControlBlock,
    load_from: *const TapeControlBlock
);
pub fn create_process(entry: u32, tape_base: u32, tape_size: u32) -> Result<u32, ProcError>;
```

## Pitfalls
- See PROC.md Section 4: context switch is 32-bit. Use `push eax` not `push rax`.
- The context switch function is `#[naked]` — compiler adds NO prologue/epilogue. Every instruction must be explicit.
- When creating a new process: `eflags` must have `IF` bit set (bit 9 = 0x200) so interrupts are enabled when process runs.
- `esp` for new process = `PROC_STACK_TOP - 4` (leave room for return address even though entry doesn't return normally).

## Test
```bash
make qemu-test TEST=proc_create TIMEOUT=15
make qemu-test TEST=proc_context_switch TIMEOUT=15
```

## On Completion
Generate: DISPATCH_STORE_001, DISPATCH_SHELL_001 (blocked on PROC + TRANS + STORE)

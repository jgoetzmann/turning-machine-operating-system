# PROC.md — Process Model Agent
> **Domain ID:** `PROC` | **Agent Slot:** Agent-09
> **TM Role:** A process is a sub-tape with a saved head position. Switching processes = repositioning the head.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

- **TCB (Tape Control Block)**: per-process state — saved registers, tape region, stack pointer
- **Context switch**: save current head position, load new head position (register save/restore)
- **PID allocator**: monotonically incrementing, simple
- **Process tape region**: each process gets a slice of `PROC_TAPE_BASE..PROC_TAPE_END`
- **Phase 1**: one process at a time (single TCB slot). No preemption.

---

## 2. File Ownership

```
kernel/src/proc/
├── mod.rs          # create_process(), destroy_process(), TCB management
├── tcb.rs          # TapeControlBlock struct (the saved head state)
└── context.rs      # context_save(), context_restore() — pure ASM
```

---

## 3. API Contracts

```rust
// proc/tcb.rs
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TapeControlBlock {
    // Saved head position (CPU registers — the head's internal state)
    pub eax: u32, pub ebx: u32, pub ecx: u32, pub edx: u32,
    pub esi: u32, pub edi: u32, pub ebp: u32, pub esp: u32,
    pub eip: u32, pub eflags: u32,
    // Process metadata
    pub pid:         u32,
    pub state:       ProcState,
    pub tape_base:   u32,   // start of this process's tape region
    pub tape_end:    u32,
    pub exit_code:   Option<u32>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ProcState { Ready, Running, Blocked, Zombie }

// proc/mod.rs
pub fn create_process(entry: u32, tape_base: u32, tape_size: u32) -> Result<u32, ProcError>;
pub fn destroy_process(pid: u32);
pub fn get_tcb(pid: u32) -> Option<&'static mut TapeControlBlock>;

// proc/context.rs
/// Save current register state into tcb.
/// Restore register state from tcb and jump to tcb.eip.
/// This is naked assembly — never call directly except from CTRL scheduler.
#[naked]
pub unsafe extern "C" fn context_switch(save_into: *mut TapeControlBlock,
                                         load_from: *const TapeControlBlock);
```

---

## 4. Critical Constraints

- **Context switch is 32-bit assembly**: push/pop `eax ebx ecx edx esi edi ebp`. No 64-bit registers.
- **User process stack**: set `esp` to `tape::regions::PROC_STACK_TOP` when creating process.
- **Process tape region**: Phase 1 has one slot (`PROC_TAPE_BASE`–`PROC_TAPE_END`, 2MB). Load process code into this region.
- **EIP for new process**: the entry point of the process ELF, loaded into the process tape region.
- **No `fork` in Phase 1** — processes are created by the kernel loading an ELF. `fork` is Phase 2.

---

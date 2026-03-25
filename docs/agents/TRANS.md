# TRANS.md — Interrupts & Syscalls Agent
> **Domain ID:** `TRANS` | **Agent Slot:** Agent-05
> **TM Role:** The transition function implementation. When an event fires, TRANS executes the response.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

TRANS implements the concrete handlers that the transition function dispatches to:

- **`int 0x80` syscall gate**: receives syscall from userspace (DPL=3 IDT gate)
- **Syscall dispatch table**: maps syscall number → handler function
- **IRQ handlers**: timer tick, keyboard, spurious — wire into IDT gates
- **Exception handlers**: #GP, #DF, #PF, #UD — all route to `MachineState::Panic`
- **`int 0x80` ABI**: 32-bit Linux i386 compatible (eax=num, ebx/ecx/edx=args)

---

## 2. File Ownership

```
kernel/src/trans/
├── mod.rs              # register_all_handlers() — called during machine_boot
├── syscall.rs          # int 0x80 entry, syscall dispatch table, all handlers
├── irq.rs              # IRQ0 (timer), IRQ1 (keyboard), IRQ7/15 (spurious)
└── exceptions.rs       # #GP, #DF, #PF, #UD → push Fault event → Panic
```

---

## 3. Syscall Table (32-bit Linux i386 ABI)

```rust
// Syscall numbers — Linux i386 compatible
pub const SYS_EXIT:    u32 = 1;
pub const SYS_FORK:    u32 = 2;   // Phase 2
pub const SYS_READ:    u32 = 3;
pub const SYS_WRITE:   u32 = 4;
pub const SYS_OPEN:    u32 = 5;
pub const SYS_CLOSE:   u32 = 6;
pub const SYS_GETPID:  u32 = 20;
pub const SYS_BRK:     u32 = 45;
pub const SYS_YIELD:   u32 = 158;  // sched_yield
```

**Phase 1 required:** `SYS_EXIT`, `SYS_WRITE` (fd=1 → serial/VGA), `SYS_GETPID`, `SYS_YIELD`
**Phase 2:** everything else

---

## 4. API Contracts

```rust
// trans/mod.rs
/// Wire all IDT gates (exceptions, IRQs, syscall). Called during machine_boot.
pub fn register_all_handlers(idt: &mut Idt);

// trans/syscall.rs
/// Entry point for int 0x80. Saves all registers, calls dispatch, restores, iret.
/// This is a naked function — compiler touches nothing.
#[naked]
pub unsafe extern "C" fn syscall_entry();

/// Dispatch by syscall number. Returns value placed in eax.
pub fn syscall_dispatch(num: u32, ebx: u32, ecx: u32, edx: u32) -> u32;

// The register frame saved on entry
#[repr(C)]
pub struct SyscallFrame {
    pub eax: u32, pub ebx: u32, pub ecx: u32, pub edx: u32,
    pub esi: u32, pub edi: u32, pub ebp: u32,
    pub eip: u32, pub cs: u32,  pub eflags: u32,
    pub esp: u32, pub ss: u32,  // only present if coming from ring 3
}
```

---

## 5. Critical Constraints

- **`int 0x80` gate is DPL=3** so userspace can invoke it. All other IDT gates are DPL=0.
- **Syscall entry is a `#[naked]` function** — do not let the compiler generate prologue/epilogue. Manually push/pop all registers. Use `pushad`/`popad` (32-bit push all).
- **After `iret` from syscall**: CS must be user code segment (0x18), so the TSS must be set up with the kernel SS:ESP for the ring 0 stack. Coordinate with HEAD.
- **Exception handlers post `Event::Fault` and do not return**: they push the event, the machine loop transitions to Panic state, and `do_panic()` halts.
- **IRQ handlers send EOI**: every IRQ handler must send `outb(0x20, 0x20)` (PIC EOI) before returning. See MISTAKE-003.
- **Timer IRQ calls `ctrl::on_timer_tick()`** which may change state. It does NOT directly reschedule — it posts `Event::TimerTick`.

---

## 6. Known Mistakes to Avoid

- **MISTAKE: Forgetting PIC EOI** — see MISTAKES.md MISTAKE-003. Every IRQ handler must send EOI.
- **MISTAKE: DPL=0 on syscall gate** — userspace executing `int 0x80` triggers #GP because DPL=0 prevents ring 3 from invoking the gate. Set the syscall gate descriptor `dpl=3`.
- **MISTAKE: Not saving all registers in syscall entry** — `pushad` saves eax/ecx/edx/ebx/esp/ebp/esi/edi. If you skip this, syscall clobbers user registers.
- **MISTAKE: Re-enabling interrupts before saving state in IRQ handler** — if `sti` fires before registers are saved, a nested IRQ corrupts the frame.

# MISTAKES.md — Mistake Registry
> **APPEND-ONLY. Never delete or modify existing entries.**
> **Read ALL entries tagged with your domain before writing any code.**

---

## Schema

```
## [MISTAKE-NNN] [DOMAIN_TAGS] — Title
**Date:** YYYY-MM-DD
**Category:** COMPILE | LOGIC | ARCH | SAFETY | SECURITY | DEADLOCK | BUILD

**Description:** What went wrong.

**Bad:**
```code```

**Good:**
```code```

**How Caught:** panic / hang / wrong output / compile error

**Impact:** What would happen if not caught.

**Prevention Rule:** ONE sentence, imperative.

**Affected Files:** list
---
```

---

## [MISTAKE-001] [HEAD] [BUILD] — 64-bit instruction in 32-bit kernel
**Date:** PRE-SEEDED
**Category:** ARCH

**Description:**
Using 64-bit register names (`rax`, `rbx`, `rsp`) or 64-bit instructions in the kernel. We target `i686-unknown-none` (32-bit). 64-bit instructions are invalid opcodes in 32-bit protected mode and will immediately trigger `#UD` (Invalid Opcode exception), which causes a triple fault before the IDT is set up, so QEMU resets silently.

**Bad:**
```rust
// In kernel code — 64-bit register reference
let stack_ptr: u64 = 0xDEADBEEF;
// Or in inline asm:
asm!("mov rax, rbx");  // WRONG — 64-bit registers don't exist in 32-bit mode
```

**Good:**
```rust
let stack_ptr: u32 = 0x0007_0000;
// In inline asm:
asm!("mov eax, ebx");  // correct — 32-bit registers
```

**How Caught:** QEMU reset immediately on boot. No serial output.

**Impact:** Silent triple fault. No error message. Very hard to debug.

**Prevention Rule:** EVERY register name in inline asm and EVERY address type in kernel code must be 32-bit (`u32`, `eax`, `esp`); any `u64` or `rax` in kernel code is a bug.

**Affected Files:** Any file in `kernel/src/`

---

## [MISTAKE-002] [HEAD] — BSS not zeroed before entering Rust
**Date:** PRE-SEEDED
**Category:** LOGIC

**Description:**
Rust assumes zero-initialized statics. Static variables with value `0` are placed in the `.bss` section. If `.bss` is not explicitly zeroed in `boot.asm` before calling `boot_init()`, those variables contain whatever garbage was in memory. This causes random failures depending on what Multiboot2 left in RAM.

**Bad:**
```asm
; boot.asm — WRONG: jumps to Rust without zeroing BSS
_start:
    mov esp, kernel_stack_top
    push ebx    ; multiboot_info
    push eax    ; magic
    call boot_init
```

**Good:**
```asm
_start:
    ; Zero BSS
    mov edi, _bss_start
    mov ecx, _bss_end
    sub ecx, edi
    xor eax, eax
    rep stosd
    ; THEN set up stack and call Rust
    mov esp, kernel_stack_top
    push ebx
    push eax
    call boot_init
```

**How Caught:** Static `Option<T>` types didn't have `None` — they had garbage `Some(...)` values.

**Impact:** Random crashes, incorrect behavior from any code that uses static variables.

**Prevention Rule:** Always zero `_bss_start..._bss_end` in `boot.asm` before the first call to any Rust function.

**Affected Files:** `kernel/src/head/boot.asm`

---

## [MISTAKE-003] [TRANS] [DEVICE] — PIC EOI not sent after IRQ
**Date:** PRE-SEEDED
**Category:** LOGIC

**Description:**
The 8259 PIC requires an End-Of-Interrupt signal (`outb(0x20, 0x20)`) after each IRQ is handled. Without it, the PIC marks the IRQ as still in-service and never sends that IRQ again. The timer fires once, then stops. The keyboard fires once, then stops.

**Bad:**
```rust
extern "x86-interrupt" fn timer_handler(_frame: &InterruptFrame) {
    device::pit::increment_tick();
    ctrl::push_event(Event::TimerTick);
    // BUG: no EOI sent — timer never fires again
}
```

**Good:**
```rust
extern "x86-interrupt" fn timer_handler(_frame: &InterruptFrame) {
    device::pit::increment_tick();
    ctrl::push_event(Event::TimerTick);
    device::pic::pic_send_eoi(0);  // IRQ0 = timer
}
```

**How Caught:** Timer fires once; machine appears to freeze in first state.

**Impact:** All timer-based events stop. Machine enters `Sleep` state and never wakes.

**Prevention Rule:** Every IRQ handler's LAST operation (before `iret`) must send PIC EOI via `pic_send_eoi(irq_number)`.

**Affected Files:** `kernel/src/trans/irq.rs`

---

## [MISTAKE-004] [TRANS] — Syscall IDT gate has DPL=0
**Date:** PRE-SEEDED
**Category:** SECURITY

**Description:**
The `int 0x80` syscall gate in the IDT must have DPL=3 (user-accessible). If it has DPL=0 (the default for exception gates), userspace executing `int 0x80` triggers a #GP (General Protection Fault) instead of entering the syscall handler. The machine panics on every syscall.

**Bad:**
```rust
// Setting all IDT gates with DPL=0, including syscall gate
idt.set_gate(0x80, syscall_entry as u32, GateFlags::INTERRUPT_GATE);
// GateFlags::INTERRUPT_GATE has DPL=0 by default
```

**Good:**
```rust
// Syscall gate MUST have DPL=3
idt.set_gate(0x80, syscall_entry as u32, GateFlags::INTERRUPT_GATE | GateFlags::DPL3);
```

**How Caught:** Every `int 0x80` from userspace triggers `#GP`. Machine transitions to `Panic`.

**Impact:** No syscalls work. Userspace cannot communicate with kernel.

**Prevention Rule:** The `int 0x80` IDT gate must use `DPL=3`; all other gates use `DPL=0`.

**Affected Files:** `kernel/src/trans/mod.rs`, `kernel/src/head/idt.rs`

---

## [MISTAKE-005] [STORE] — FAT12 cluster 1.5-byte packing got even/odd wrong
**Date:** PRE-SEEDED
**Category:** LOGIC

**Description:**
FAT12 packs two 12-bit cluster entries into 3 bytes. The formula for even vs. odd entries is different. Getting them swapped makes every cluster chain follow garbage pointers, reading random tape sectors as file data.

**Bad:**
```rust
// Swapped even/odd — WRONG
fn fat12_entry(fat: &[u8], n: usize) -> u16 {
    let idx = n + n / 2;
    if n % 2 == 0 {
        // WRONG: this is the odd formula
        ((fat[idx] as u16 >> 4) | (fat[idx+1] as u16) << 4)
    } else {
        // WRONG: this is the even formula
        fat[idx] as u16 | ((fat[idx+1] as u16 & 0x0F) << 8)
    }
}
```

**Good:**
```rust
fn fat12_entry(fat: &[u8], n: usize) -> u16 {
    let idx = n + n / 2;
    if n % 2 == 0 {
        // Even: low byte | low nibble of high byte
        fat[idx] as u16 | ((fat[idx+1] as u16 & 0x0F) << 8)
    } else {
        // Odd: high nibble of low byte | high byte
        (fat[idx] as u16 >> 4) | (fat[idx+1] as u16) << 4
    }
}
```

**How Caught:** File contents are garbage. Cluster chain points to sector 0 or wraps.

**Impact:** All file reads return corrupted data. Files cannot be opened correctly.

**Prevention Rule:** Write a standalone unit test for `fat12_entry` with known byte patterns BEFORE integrating with disk I/O.

**Affected Files:** `kernel/src/store/fat12.rs`

---

## [MISTAKE-006] [HEAD] — Segment registers not reloaded after GDT load
**Date:** PRE-SEEDED
**Category:** ARCH

**Description:**
After `lgdt`, the CPU caches the old GDT values in its segment descriptor cache. CS retains the old code segment value. To flush the cache, you MUST do a far jump to reload CS. DS/ES/FS/GS/SS must be explicitly loaded with `mov` instructions.

**Bad:**
```asm
lgdt [gdt_descriptor]
; BUG: CS still has old segment, far jump not performed
jmp .continue
.continue:
    mov ax, 0x10  ; only loads DS, not CS
```

**Good:**
```asm
lgdt [gdt_descriptor]
jmp 0x08:.reload_cs    ; far jump to reload CS with kernel code segment
.reload_cs:
    mov ax, 0x10       ; kernel data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
```

**How Caught:** Random #GP faults immediately after GDT load.

**Impact:** CPU executes with stale segment descriptors. #GP on first privileged operation.

**Prevention Rule:** After every `lgdt`, perform a far jump to reload CS, then explicitly reload DS/ES/FS/GS/SS.

**Affected Files:** `kernel/src/head/boot.asm`, `kernel/src/head/gdt.rs`

---

## [MISTAKE-007] [DISPLAY] — Non-volatile write to VGA buffer optimized away
**Date:** PRE-SEEDED
**Category:** LOGIC

**Description:**
The VGA buffer at `0xB8000` is a memory-mapped I/O region. If you write to it with a regular pointer dereference (not `write_volatile`), the compiler may determine "nothing reads this memory" and eliminate the write entirely. Screen remains blank with no error.

**Bad:**
```rust
let vga = 0xB8000 as *mut u16;
unsafe { *vga = 0x0748 };  // 'H' with grey-on-black — may be optimized out
```

**Good:**
```rust
use core::ptr::write_volatile;
let vga = tape::regions::VGA_BASE as *mut u16;
unsafe { write_volatile(vga, 0x0748) };  // volatile — compiler cannot remove this
```

**How Caught:** Screen blank. Serial output works. Adding `volatile` fixes it.

**Impact:** No display output. Hard to debug since serial still works.

**Prevention Rule:** All reads and writes to the VGA tape region MUST use `tape::tape_read`/`tape::tape_write` (which wrap `read_volatile`/`write_volatile`).

**Affected Files:** `kernel/src/display/vga.rs`

---

*— Append new entries below this line. Never modify entries above. —*

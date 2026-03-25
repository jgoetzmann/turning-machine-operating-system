# DISPATCH_HEAD_001 — Multiboot2 Entry + 32-bit GDT + Serial Hello

## Header
**Domain:** HEAD | **Agent:** Agent-02 | **Status:** PENDING
**Priority:** CRITICAL | **Created:** PROJECT_START
**Branch:** agent/head-multiboot2-entry

## Prerequisites
- [ ] DISPATCH_BUILD_001 COMPLETE

## Context Snapshot
None — load `docs/agents/HEAD.md` only.

## Objective
QEMU `-kernel` loads the 32-bit ELF. The kernel validates Multiboot2 magic, sets up its own flat GDT, initializes COM1 serial, zeros BSS, and prints `HEAD: tape machine booting` to serial. Test `head_boot_basic` passes.

## Acceptance Criteria
- [ ] `boot.asm`: Multiboot2 header (magic `0xE85250D6`), checksum
- [ ] `boot.asm`: validates `eax == 0x36d76289` on entry, halts if wrong
- [ ] `boot.asm`: zeros `_bss_start..._bss_end` before calling Rust
- [ ] `boot.asm`: sets `esp` to `KERNEL_STACK_TOP` before calling Rust
- [ ] `gdt.rs`: 6-entry GDT (null, kernel code/data, user code/data, TSS placeholder)
- [ ] After `lgdt`: far jump reloads CS, then reload DS/ES/FS/GS/SS
- [ ] `serial.rs`: COM1 at `0x3F8`, 115200 baud, 8N1
- [ ] `serial_print!` and `serial_println!` macros work
- [ ] Prints `HEAD: tape machine booting` on serial
- [ ] `make qemu-test TEST=head_boot_basic` passes
- [ ] `[TEST PASS: head_boot_basic]` appears on serial output
- [ ] **All code is 32-bit: `u32` addresses, `eax/esp` registers, no `rax`/`u64`**

## Files To Touch
- CREATE: `kernel/src/head/boot.asm` — multiboot2 header + `_start`
- CREATE: `kernel/src/head/gdt.rs` — flat GDT
- CREATE: `kernel/src/head/idt.rs` — empty IDT (TRANS fills gates; HEAD just allocates + lidt)
- CREATE: `kernel/src/head/serial.rs` — COM1 driver
- CREATE: `kernel/src/head/memmap.rs` — stub (parse multiboot2 memory map later)
- CREATE: `kernel/src/head/mod.rs` — `pub fn boot_init(magic: u32, info: u32) -> !`
- MODIFY: `kernel/src/main.rs` — add `extern "C" fn _start` link to boot.asm
- CREATE: `tests/integration/head_boot_basic.rs` — prints sentinels to serial
- MODIFY: `kernel/linker.ld` — add `.stack` section, KEEP multiboot2 section

## API Spec
```rust
// kernel/src/head/mod.rs
pub fn boot_init(multiboot_magic: u32, multiboot_info_phys: u32) -> !;

// kernel/src/head/serial.rs  
pub fn serial_init();
pub fn serial_write_byte(b: u8);
pub fn serial_write_str(s: &str);
#[macro_export] macro_rules! serial_print { ... }
#[macro_export] macro_rules! serial_println { ... }
```

## Pitfalls
- See MISTAKE-002: Zero BSS BEFORE calling boot_init. Rust statics are garbage otherwise.
- See MISTAKE-006: After `lgdt`, far jump to reload CS. Then reload DS/ES/FS/GS/SS explicitly.
- See MISTAKE-001: All registers in boot.asm are 32-bit (eax, esp, etc.)
- Multiboot2 header must be within first 8KB of ELF. `KEEP(*(.multiboot2))` in linker script.
- Initial stack: `esp = 0x00007000` (below kernel at 1MB, in low memory). Safe to use before TAPE.

## Test
```bash
make qemu-test TEST=head_boot_basic TIMEOUT=10
```
Expected serial output:
```
HEAD: tape machine booting
[TEST START: head_boot_basic]
[TEST PASS: head_boot_basic]
```

## On Completion
1. Check all criteria
2. Append ITER-002 to ITERATIONS.md
3. Status: COMPLETE, delete lock
4. PR: `feat(head): multiboot2 entry + GDT + serial — DISPATCH_HEAD_001`
5. Generate DISPATCH_TAPE_001, DISPATCH_DEVICE_001, DISPATCH_DISPLAY_001

## Related Cards
- DISPATCH_BUILD_001 (blocked on this)
- DISPATCH_TAPE_001, DISPATCH_DEVICE_001, DISPATCH_DISPLAY_001 (unblocked by this)

# DISPATCH_DEVICE_001 — PIC + PIT Timer + PS/2 Keyboard

## Header
**Domain:** DEVICE | **Agent:** Agent-06 | **Status:** PENDING
**Priority:** CRITICAL | **Created:** PROJECT_START
**Branch:** agent/device-pic-pit-keyboard

## Prerequisites
- [ ] DISPATCH_HEAD_001 COMPLETE — need GDT + IDT stub + serial

## Context Snapshot
None — load `docs/agents/DEVICE.md` only.

## Objective
Initialize the 8259 PIC (remapped to 0x20/0x28), the PIT at 100Hz, and the PS/2 keyboard driver. After this task, IRQ0 fires at 100Hz and IRQ1 fires on each keypress. TRANS wires the handlers; DEVICE provides the hardware init and the `pic_send_eoi`, `pit_get_ticks`, and `keyboard_read_char` APIs.

## Acceptance Criteria
- [ ] `pic_init(0x20, 0x28)` remaps PIC1→0x20, PIC2→0x28
- [ ] `pic_send_eoi(irq)` sends EOI to PIC1, and also PIC2 for IRQs 8–15
- [ ] `pic_is_spurious(irq)` checks ISR for IRQ7 and IRQ15
- [ ] `pit_init(100)` sets PIT channel 0 to 100Hz (divisor 11931)
- [ ] `pit_get_ticks()` returns a monotonically increasing `u64` tick counter
- [ ] `keyboard_init()` clears keyboard buffer, enables IRQ1
- [ ] `keyboard_handle_irq()` reads `0x60`, converts scancode set 1 → ASCII
- [ ] `keyboard_read_char()` returns `Option<char>` (non-blocking)
- [ ] `make qemu-test TEST=device_timer` passes (tick counter increments)
- [ ] `make qemu-test TEST=device_keyboard` passes (keypress → char)
- [ ] **All port numbers as constants, all registers 32-bit**

## Files To Touch
- CREATE: `kernel/src/device/pic.rs`
- CREATE: `kernel/src/device/pit.rs`
- CREATE: `kernel/src/device/keyboard.rs`
- CREATE: `kernel/src/device/mod.rs` — `pub fn init_all_devices()`
- CREATE: `tests/integration/device_timer.rs`
- CREATE: `tests/integration/device_keyboard.rs`

## API Spec
```rust
pub fn pic_init(offset1: u8, offset2: u8);
pub fn pic_send_eoi(irq: u8);
pub fn pic_is_spurious(irq: u8) -> bool;
pub fn pit_init(freq_hz: u32);
pub fn pit_get_ticks() -> u64;
pub fn keyboard_handle_irq();
pub fn keyboard_read_char() -> Option<char>;
```

## Pitfalls
- See MISTAKE-003: `pic_send_eoi` must be called at end of every IRQ handler (by TRANS, not here — but you must provide it)
- PIC remapping MUST happen before `sti`. Default IRQ vectors 0x00–0x0F collide with CPU exceptions.
- Spurious IRQ7: check PIC1 ISR (In-Service Register) before treating IRQ7 as real. Port 0x20 with OCW3 command 0x0B reads ISR.
- PIT divisor for 100Hz: `1193182 / 100 = 11931`. Load low byte then high byte into port 0x40 after sending mode command to 0x43.
- Scancode set 1: key-down is `0x01`–`0x58`. Key-up is `0x81`–`0xD8` (key-down + 0x80). Ignore key-up for Phase 1.

## Test
```bash
make qemu-test TEST=device_timer TIMEOUT=15
make qemu-test TEST=device_keyboard TIMEOUT=15
```

## On Completion
1. Append ITER-NNN to ITERATIONS.md
2. Status: COMPLETE, delete lock
3. PR: `feat(device): PIC + PIT + keyboard — DISPATCH_DEVICE_001`
4. Generate DISPATCH_DEVICE_002 (ATA disk — needed by STORE)

## Related Cards
- DISPATCH_CTRL_001 (parallel — CTRL needs TimerTick events, DEVICE provides them)
- DISPATCH_TRANS_001 (TRANS wires IRQ handlers using DEVICE's init)
- DISPATCH_DEVICE_002 (ATA — blocked on this)

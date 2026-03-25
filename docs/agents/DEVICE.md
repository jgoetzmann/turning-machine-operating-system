# DEVICE.md — Hardware I/O Devices Agent
> **Domain ID:** `DEVICE` | **Agent Slot:** Agent-06
> **TM Role:** The tape input/output devices. Keyboard writes symbols TO the tape. Serial reads symbols FROM the tape. Timer drives the machine's clock.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

DEVICE owns all direct hardware I/O:

- **8259 PIC**: interrupt controller — route IRQs to IDT vectors
- **PIT (8253 timer)**: system tick at 100Hz — drives the machine clock
- **PS/2 Keyboard**: converts keypresses to tape symbols (ASCII)
- **Serial UART (16550)**: full COM1 driver — used for debug and test output
- **ATA PIO disk**: block device — read/write 512B sectors (used by STORE domain)

The HEAD domain has a minimal serial for early boot. This domain replaces it with the full driver.

---

## 2. File Ownership

```
kernel/src/device/
├── mod.rs              # init_all_devices() — called during machine_boot
├── pic.rs              # 8259 PIC init, EOI, mask/unmask
├── pit.rs              # PIT init (100Hz), tick counter
├── keyboard.rs         # PS/2 keyboard driver, scancode → ASCII
└── ata.rs              # ATA PIO — read_sector(lba, buf), write_sector(lba, buf)
```

---

## 3. API Contracts

```rust
// device/mod.rs
pub fn init_all_devices();

// device/pic.rs
pub fn pic_init(offset1: u8, offset2: u8);  // remap to 0x20/0x28
pub fn pic_send_eoi(irq: u8);
pub fn pic_mask(irq: u8);
pub fn pic_unmask(irq: u8);
pub fn pic_is_spurious(irq: u8) -> bool;    // check ISR for IRQ7/15

// device/pit.rs
pub fn pit_init(freq_hz: u32);              // set frequency via divisor
pub fn pit_get_ticks() -> u64;             // global tick counter (u64, wraps in ~6M years)
pub fn pit_sleep_ticks(ticks: u64);        // busy-wait N ticks

// device/keyboard.rs
pub fn keyboard_init();
pub fn keyboard_handle_irq();              // called from TRANS IRQ1 handler
pub fn keyboard_read_char() -> Option<char>; // non-blocking

// device/ata.rs — consumed by STORE
pub fn ata_init() -> bool;                 // returns false if no disk present
pub fn ata_read_sector(lba: u32, buf: &mut [u8; 512]) -> Result<(), AtaError>;
pub fn ata_write_sector(lba: u32, buf: &[u8; 512]) -> Result<(), AtaError>;
pub fn ata_identify() -> Option<AtaIdentify>;
```

---

## 4. PIC Configuration

After `pic_init(0x20, 0x28)`:
- IRQ0 (timer) → IDT vector 0x20
- IRQ1 (keyboard) → IDT vector 0x21
- IRQ7 → IDT vector 0x27 (spurious — check `pic_is_spurious(7)`)
- IRQ15 → IDT vector 0x2F (spurious — check `pic_is_spurious(15)`)

Mask all IRQs except 0 (timer) and 1 (keyboard) on init. Unmask others when their domain registers a handler.

---

## 5. Critical Constraints

- **PIC remapping is mandatory**: default IRQ vectors (0x00–0x0F) collide with CPU exceptions. Remap PIC to 0x20/0x28 before enabling interrupts.
- **PIT at 100Hz**: divisor = 1193182 / 100 = 11931. Load into channel 0 with mode 3 (square wave). 100Hz = 10ms tick — a reasonable TM clock cycle.
- **Scancode set 1**: QEMU sends PS/2 scan code set 1. Parse it in `keyboard.rs`. Build a simple lookup table.
- **ATA PIO is synchronous and slow**: that's fine for a TM OS. We're not optimizing for throughput.
- **Port I/O requires privilege (ring 0)**: all port I/O happens in kernel mode. Never expose port I/O to userspace.
- **All MMIO reads/writes use `tape_read`/`tape_write`**: see TAPE domain.

---

## 6. Known Mistakes to Avoid

- **MISTAKE: Enabling IRQs before PIC is remapped** — exceptions will be misrouted. `pic_init()` must come before `sti`.
- **MISTAKE: Not handling spurious IRQ7** — the PIC sends spurious IRQ7 sometimes. If you handle it like a real IRQ, send EOI to PIC1 only. If you read the PIC's ISR and bit 7 is not set, it's spurious — DO NOT send EOI.
- **MISTAKE: ATA polling timeout missing** — if no disk is present, ATA status register reads 0xFF. Poll with a timeout counter, don't spin forever.
- **MISTAKE: PIT divisor = 0** — divisor 0 means 65536 (wraps), giving ~18.2Hz. Set the correct divisor for 100Hz.

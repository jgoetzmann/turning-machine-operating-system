# DISPATCH_DEVICE_002 — ATA PIO Disk Driver

## Header
**Domain:** DEVICE | **Agent:** Agent-06 | **Status:** PENDING
**Priority:** HIGH | **Created:** PROJECT_START
**Branch:** agent/device-ata

## Prerequisites
- [ ] DISPATCH_DEVICE_001 COMPLETE

## Context Snapshot
`context-snapshots/` — check for latest DEVICE snapshot after DEVICE_001 completes.

## Objective
Implement ATA PIO mode disk driver. Read and write 512-byte sectors by LBA address. After this task, STORE domain can read the FAT12 disk image from sector 0.

## Acceptance Criteria
- [ ] `ata_init()` detects disk presence (status != 0xFF), returns false if no disk
- [ ] `ata_read_sector(lba, buf)` reads 512 bytes from LBA sector into `buf`
- [ ] `ata_write_sector(lba, buf)` writes 512 bytes from `buf` to LBA sector
- [ ] Polling loop has a timeout counter (stops after 100,000 iterations max)
- [ ] Returns `Err(AtaError::Timeout)` if disk doesn't respond
- [ ] Returns `Err(AtaError::Error)` if status register has error bit set
- [ ] `make qemu-test TEST=device_ata` passes (read sector 0 of fat12.img, check BPB signature `0x55 0xAA`)
- [ ] **LBA address is `u32`. Sector buffer is `[u8; 512]`.**

## Files To Touch
- CREATE: `kernel/src/device/ata.rs`
- MODIFY: `kernel/src/device/mod.rs` — add `pub use ata::*`
- CREATE: `tests/integration/device_ata.rs`
- CREATE: `tools/make_fat12.sh` — script to create 1.44MB FAT12 test image
- CREATE: `tests/fixtures/fat12.img` — via `make_fat12.sh` (add to git via `git lfs` or just commit it)

## API Spec
```rust
#[derive(Debug)]
pub enum AtaError { Timeout, Error, NoDisk }

pub fn ata_init() -> bool;
pub fn ata_read_sector(lba: u32, buf: &mut [u8; 512]) -> Result<(), AtaError>;
pub fn ata_write_sector(lba: u32, buf: &[u8; 512]) -> Result<(), AtaError>;
```

## Pitfalls
- ATA primary bus: data port `0x1F0`, status `0x1F7`, command `0x1F7`
- BSY bit (bit 7) must be clear before sending command. Poll status with timeout.
- DRQ bit (bit 3) must be set before reading data words. Another polling wait.
- Read 256 16-bit words (512 bytes) using `insw` or a loop of `inw(0x1F0)`.
- If status register reads `0xFF`, no drive present. `ata_init()` returns false.
- LBA28 addressing: send LBA bits 0-7 to `0x1F3`, bits 8-15 to `0x1F4`, bits 16-23 to `0x1F5`, bits 24-27 | `0xE0` to `0x1F6`.

## Test
```bash
make make-disk        # creates tests/fixtures/fat12.img
make qemu-test TEST=device_ata TIMEOUT=15
```

---

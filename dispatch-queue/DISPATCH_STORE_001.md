# DISPATCH_STORE_001 — FAT12 Filesystem

## Header
**Domain:** STORE | **Agent:** Agent-08 | **Status:** PENDING
**Priority:** HIGH | **Created:** PROJECT_START
**Branch:** agent/store-fat12

## Prerequisites
- [ ] DISPATCH_DEVICE_002 COMPLETE — need `ata_read_sector`
- [ ] DISPATCH_TAPE_001 COMPLETE — need heap + `FS_CACHE_BASE` region

## Context Snapshot
None — load `docs/agents/STORE.md` only.

## Objective
Mount a FAT12 filesystem from the ATA disk. List the root directory. Read a file by name. After this task, the kernel can load userspace program ELFs from disk into the process tape region.

## Acceptance Criteria
- [ ] `store_init()` reads BPB from sector 0, validates FAT12 signature `0x55 0xAA`
- [ ] FAT12 cluster entry function handles even/odd packing correctly (unit test FIRST)
- [ ] `list_root()` returns directory entries from the fixed-size root directory
- [ ] `open("HELLO.TXT")` returns a `FileHandle` if file exists
- [ ] `read(fh, buf)` reads file content correctly, returns byte count
- [ ] `close(fh)` releases the handle
- [ ] Sector cache in `tape::regions::FS_CACHE_BASE` buffers recently-read sectors (at least 16 sectors)
- [ ] `make qemu-test TEST=store_fat12` passes (mount, list root, read HELLO.TXT)
- [ ] **All LBA addresses and cluster numbers are `u32` or `u16`. No `u64`.**

## Files To Touch
- CREATE: `kernel/src/store/fat12.rs`
- CREATE: `kernel/src/store/mod.rs`
- MODIFY: `tests/fixtures/fat12.img` — add a `HELLO.TXT` file with content "HELLO TAPE\n"
- MODIFY: `tools/make_fat12.sh` — add HELLO.TXT creation step
- CREATE: `tests/integration/store_fat12.rs`

## API Spec
```rust
pub fn store_init() -> Result<(), StoreError>;
pub fn open(name: &str) -> Result<FileHandle, StoreError>;
pub fn read(fh: FileHandle, buf: &mut [u8]) -> Result<usize, StoreError>;
pub fn write(fh: FileHandle, buf: &[u8]) -> Result<usize, StoreError>;
pub fn close(fh: FileHandle);
pub fn list_root() -> Result<heapless::Vec<DirEntry, 16>, StoreError>;
```

## Pitfalls
- See MISTAKE-005: FAT12 even/odd cluster packing. Write `fat12_entry()` unit test BEFORE anything else.
- See STORE.md: End-of-chain is ANY value `>= 0xFF8`, not just `0xFFF`. Check with `val >= 0xFF8`.
- See STORE.md: Root dir sector count = `(root_entry_count * 32 + bytes_per_sector - 1) / bytes_per_sector`
- See STORE.md: First data sector = `reserved + (num_fats * fat_sectors) + root_dir_sectors`
- FAT12 filenames are 8.3 format, uppercase, space-padded. "HELLO.TXT" is stored as `"HELLO   TXT"`.
- Cluster 2 = first data cluster. `lba = first_data_sector + (cluster - 2) * sectors_per_cluster`

## Test
```bash
make make-disk        # creates fat12.img with HELLO.TXT
make qemu-test TEST=store_fat12 TIMEOUT=20
```

## On Completion
Generate: DISPATCH_SHELL_001

---

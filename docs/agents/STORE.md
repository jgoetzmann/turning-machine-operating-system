# STORE.md — Filesystem Agent
> **Domain ID:** `STORE` | **Agent Slot:** Agent-08
> **TM Role:** The persistent tape. Write symbols to disk so they survive machine halt and resume.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

FAT12 filesystem over ATA PIO disk. That's it.

- **FAT12**: the filesystem of 1.44MB floppy disks. Simple. Matches TM philosophy — the tape is small.
- **Boot sector parsing**: read BPB (BIOS Parameter Block) from sector 0
- **Cluster chain**: follow FAT12 entries (1.5 bytes per entry — the fun part)
- **File read/write**: `open`, `read`, `write`, `close` as tape region operations
- **Root directory**: fixed-size directory, 16 entries max (FAT12 standard)

---

## 2. File Ownership

```
kernel/src/store/
├── mod.rs          # store_init(), open(), read(), write(), close()
├── fat12.rs        # FAT12 BPB, cluster chain, directory entries
└── ata.rs          # Thin wrapper calling device::ata — NOT owned by STORE, just imported
```

---

## 3. API Contracts

```rust
// store/mod.rs
pub fn store_init() -> Result<(), StoreError>;

pub struct FileHandle(u32);  // opaque handle

pub fn open(path: &str, write: bool) -> Result<FileHandle, StoreError>;
pub fn read(fh: FileHandle, buf: &mut [u8]) -> Result<usize, StoreError>;
pub fn write(fh: FileHandle, buf: &[u8]) -> Result<usize, StoreError>;
pub fn close(fh: FileHandle);
pub fn list_root() -> Result<[DirEntry; 16], StoreError>;

pub struct DirEntry {
    pub name: [u8; 8],
    pub ext:  [u8; 3],
    pub size: u32,
    pub first_cluster: u16,
}
```

---

## 4. FAT12 Notes (Read Carefully)

FAT12 has 12-bit cluster entries packed in 1.5 bytes. Two entries share 3 bytes:

```
Entry N (even):  byte[0] | (byte[1] & 0x0F) << 8
Entry N (odd):   (byte[1] >> 4) | byte[2] << 4
```

End-of-chain: any value >= `0xFF8` (0xFF8 through 0xFFF).
Free cluster: `0x000`.
Cluster 0 and 1 are reserved. Valid data starts at cluster 2.

**This is the #1 source of bugs in STORE.** See MISTAKES.md.

---

## 5. Critical Constraints

- **FAT12 only** — do not generalize to FAT16/FAT32. INV-09.
- **Disk image for testing**: `tests/fixtures/fat12.img` — a 1.44MB FAT12 image. Use `tools/make_fat12.sh` to create it.
- **Filesystem cache is a tape region**: `tape::regions::FS_CACHE_BASE`–`FS_CACHE_END`. Buffer disk sectors here.
- **No concurrent access** — single process, single head, no locking needed for Phase 1.

---

## 6. Known Mistakes to Avoid

- **MISTAKE: FAT12 1.5-byte packing** — getting even/odd cluster index wrong corrupts every file. See above. Write a unit test for this FIRST, before any integration.
- **MISTAKE: Root directory entry count** — FAT12 floppy has exactly 224 root directory entries (for 1.44MB). Do not assume 16. Read from BPB.
- **MISTAKE: First data sector calculation** — `first_data_sector = reserved_sectors + (num_fats * fat_size_sectors) + root_dir_sectors`. Off by one here = all file data reads from wrong sector.

---

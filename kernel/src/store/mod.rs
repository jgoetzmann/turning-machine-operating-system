// kernel/src/store/mod.rs
// STORE domain — FAT12 filesystem
// Agent: Agent-08
// Read docs/agents/STORE.md before modifying.

pub mod fat12;  // FAT12 BPB, cluster chain, directory entries, file I/O

#[derive(Debug)]
pub enum StoreError {
    NoDisk,
    BadSignature,
    NotFound,
    ReadError,
    WriteError,
    BufferTooSmall,
    InvalidCluster,
}

/// Opaque file handle.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FileHandle(u32);

static mut MOUNTED: bool = false;

/// Mount the FAT12 filesystem from the primary ATA disk.
/// Must be called after device::ata::ata_init() succeeds.
pub fn store_init() -> Result<(), StoreError> {
    // TODO (Agent-08): implement FAT12 mount
    // 1. Read sector 0 (boot sector / BPB)
    // 2. Validate signature bytes [510]=0x55, [511]=0xAA
    // 3. Parse BPB fields: bytes_per_sector, sectors_per_cluster,
    //    reserved_sectors, num_fats, root_entry_count, fat_size_sectors
    // 4. Compute: first_fat_sector, first_root_sector, first_data_sector
    // 5. Set MOUNTED = true
    crate::serial_println!("STORE: init (TODO — Agent-08)");
    Ok(())
}

/// Open a file by name (8.3 format, e.g. "HELLO   TXT").
pub fn open(_name: &str, _write: bool) -> Result<FileHandle, StoreError> {
    // TODO (Agent-08)
    Err(StoreError::NotFound)
}

/// Read bytes from an open file.
pub fn read(_fh: FileHandle, _buf: &mut [u8]) -> Result<usize, StoreError> {
    // TODO (Agent-08)
    Err(StoreError::ReadError)
}

/// Write bytes to an open file.
pub fn write(_fh: FileHandle, _buf: &[u8]) -> Result<usize, StoreError> {
    // TODO (Agent-08)
    Err(StoreError::WriteError)
}

/// Close a file handle.
pub fn close(_fh: FileHandle) {
    // TODO (Agent-08)
}

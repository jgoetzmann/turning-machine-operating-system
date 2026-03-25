// kernel/src/store/fat12.rs
// FAT12 filesystem implementation.
// Agent: STORE (Agent-08)
//
// WRITE THE UNIT TEST FOR fat12_entry() FIRST before any other code.
// See MISTAKE-005 in docs/MISTAKES.md — the even/odd packing is the #1 source of bugs.
//
// FAT12 cluster entry packing (1.5 bytes per entry):
//   Two entries share 3 bytes. Given cluster index n:
//     byte_offset = n + n/2  (integer division)
//   If n is EVEN:
//     entry = bytes[offset] | ((bytes[offset+1] & 0x0F) << 8)
//   If n is ODD:
//     entry = (bytes[offset] >> 4) | (bytes[offset+1] << 4)
//
// End-of-chain: ANY value >= 0xFF8 (not just 0xFFF — there are 8 valid EOF values)
// Free cluster: 0x000
// Reserved: clusters 0 and 1 (first data cluster = 2)

/// BIOS Parameter Block — parsed from the FAT12 boot sector.
#[derive(Debug, Default)]
pub struct Bpb {
    pub bytes_per_sector:     u16,  // Always 512 for us
    pub sectors_per_cluster:  u8,
    pub reserved_sectors:     u16,  // Sectors before first FAT
    pub num_fats:             u8,   // Number of FAT copies (usually 2)
    pub root_entry_count:     u16,  // Number of root directory entries
    pub total_sectors:        u16,  // Total sectors on disk
    pub fat_size_sectors:     u16,  // Sectors per FAT table
    // Derived fields (computed from above)
    pub first_fat_sector:     u32,
    pub first_root_sector:    u32,
    pub root_dir_sectors:     u32,
    pub first_data_sector:    u32,
}

impl Bpb {
    /// Parse BPB from a 512-byte boot sector.
    /// Returns Err if signature bytes at [510:511] are not 0x55 0xAA.
    pub fn parse(sector: &[u8; 512]) -> Result<Self, super::StoreError> {
        // Validate boot sector signature
        if sector[510] != 0x55 || sector[511] != 0xAA {
            return Err(super::StoreError::BadSignature);
        }

        let bytes_per_sector    = u16::from_le_bytes([sector[11], sector[12]]);
        let sectors_per_cluster = sector[13];
        let reserved_sectors    = u16::from_le_bytes([sector[14], sector[15]]);
        let num_fats            = sector[16];
        let root_entry_count    = u16::from_le_bytes([sector[17], sector[18]]);
        let total_sectors       = u16::from_le_bytes([sector[19], sector[20]]);
        let fat_size_sectors    = u16::from_le_bytes([sector[22], sector[23]]);

        // Compute derived values
        let first_fat_sector = reserved_sectors as u32;
        let first_root_sector = first_fat_sector
            + (num_fats as u32 * fat_size_sectors as u32);
        let root_dir_sectors = ((root_entry_count as u32 * 32)
            + (bytes_per_sector as u32 - 1)) / bytes_per_sector as u32;
        let first_data_sector = first_root_sector + root_dir_sectors;

        Ok(Bpb {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            root_entry_count,
            total_sectors,
            fat_size_sectors,
            first_fat_sector,
            first_root_sector,
            root_dir_sectors,
            first_data_sector,
        })
    }

    /// Convert a cluster number to its LBA sector address.
    pub fn cluster_to_lba(&self, cluster: u16) -> u32 {
        self.first_data_sector + (cluster as u32 - 2) * self.sectors_per_cluster as u32
    }
}

/// Read a FAT12 cluster entry from the FAT table bytes.
///
/// # Arguments
/// * `fat` - The FAT table bytes (fat_size_sectors * 512 bytes)
/// * `n`   - Cluster number to look up
///
/// # Returns
/// The FAT12 entry value:
///   0x000        = free cluster
///   0x002–0xFEF  = next cluster in chain
///   0xFF0–0xFF6  = reserved
///   0xFF7        = bad cluster
///   0xFF8–0xFFF  = end of chain (EOF)
///
/// WRITE A UNIT TEST FOR THIS FUNCTION FIRST. See MISTAKE-005.
pub fn fat12_entry(fat: &[u8], n: u16) -> u16 {
    let n = n as usize;
    let byte_offset = n + n / 2;

    // Guard against out-of-bounds FAT access
    if byte_offset + 1 >= fat.len() {
        return 0xFF8;  // treat as EOF rather than panic
    }

    if n % 2 == 0 {
        // Even cluster: low byte fully used, high nibble from next byte
        fat[byte_offset] as u16 | ((fat[byte_offset + 1] as u16 & 0x0F) << 8)
    } else {
        // Odd cluster: high nibble of current byte + full next byte
        (fat[byte_offset] as u16 >> 4) | (fat[byte_offset + 1] as u16) << 4
    }
}

/// Check if a FAT12 entry value represents end-of-chain.
/// End-of-chain: ANY value in range [0xFF8, 0xFFF].
#[inline]
pub fn is_eof(entry: u16) -> bool {
    entry >= 0xFF8
}

/// Check if a FAT12 entry represents a free cluster.
#[inline]
pub fn is_free(entry: u16) -> bool {
    entry == 0x000
}

/// A 32-byte FAT12 directory entry.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct DirEntry {
    pub name:     [u8; 8],   // filename, space-padded, uppercase
    pub ext:      [u8; 3],   // extension, space-padded, uppercase
    pub attrs:    u8,        // file attributes
    _reserved:    [u8; 10], // reserved + time fields (ignored)
    pub cluster:  u16,       // first cluster number
    pub size:     u32,       // file size in bytes
}

impl DirEntry {
    /// Returns true if this directory entry slot is empty (never used).
    pub fn is_empty(&self) -> bool { self.name[0] == 0x00 }

    /// Returns true if this entry has been deleted.
    pub fn is_deleted(&self) -> bool { self.name[0] == 0xE5 }

    /// Returns true if this is a file (not a directory or volume label).
    pub fn is_file(&self) -> bool {
        self.attrs & 0x10 == 0 && self.attrs & 0x08 == 0
    }

    /// Returns the name as a fixed-size array in "NAME    EXT" format.
    pub fn name_ext(&self) -> [u8; 11] {
        let mut buf = [b' '; 11];
        buf[..8].copy_from_slice(&self.name);
        buf[8..11].copy_from_slice(&self.ext);
        buf
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────
// These run on the host (cargo test) without QEMU.
// WRITE THESE BEFORE implementing anything else in this file.

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify even cluster entry extraction.
    /// Three-byte sequence [0xAB, 0xCD, 0x00]:
    ///   cluster 0 (even) = 0xAB | ((0xCD & 0x0F) << 8) = 0xDAB
    #[test]
    fn test_fat12_entry_even() {
        let fat = [0xAB, 0xCD, 0x00];
        assert_eq!(fat12_entry(&fat, 0), 0xDAB);
    }

    /// Verify odd cluster entry extraction.
    /// Three-byte sequence [0xAB, 0xCD, 0x00]:
    ///   cluster 1 (odd)  = (0xAB >> 4) | (0xCD << 4) = 0x0A | 0xD0 = 0xCDA
    ///   Wait, let me recompute: (0xAB >> 4) = 0x0A, (0xCD << 4) = 0xD0
    ///   (But this is u8 math... we need u16)
    ///   (0xAB as u16 >> 4) = 0x000A
    ///   (0xCD as u16) << 4 = 0x0CD0
    ///   result = 0x0CDA
    #[test]
    fn test_fat12_entry_odd() {
        let fat = [0xAB, 0xCD, 0x00];
        assert_eq!(fat12_entry(&fat, 1), 0xCDA);
    }

    #[test]
    fn test_is_eof() {
        assert!(is_eof(0xFF8));
        assert!(is_eof(0xFFF));
        assert!(is_eof(0xFFE));
        assert!(!is_eof(0xFF7));  // bad cluster, not EOF
        assert!(!is_eof(0x003));  // normal cluster
    }

    #[test]
    fn test_bpb_signature() {
        let mut sector = [0u8; 512];
        // Invalid signature
        assert!(Bpb::parse(&sector).is_err());
        // Valid signature
        sector[510] = 0x55;
        sector[511] = 0xAA;
        assert!(Bpb::parse(&sector).is_ok());
    }
}

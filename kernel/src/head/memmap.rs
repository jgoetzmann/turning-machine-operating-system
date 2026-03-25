// kernel/src/head/memmap.rs
// Parse Multiboot2 memory map → TapeRegion list.
// Agent: HEAD (Agent-02)
//
// Multiboot2 provides a memory map tag telling us which physical
// address ranges are usable RAM vs BIOS-reserved vs bad.
// We store this as a static array of TapeRegions for the TAPE domain.

/// A physical tape region — one entry from the Multiboot2 memory map.
#[derive(Copy, Clone, Debug)]
pub struct TapeRegion {
    pub base:   u32,             // Physical start address
    pub length: u32,             // Length in bytes
    pub kind:   TapeRegionKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TapeRegionKind {
    Available        = 1,  // Usable RAM — TAPE domain may allocate here
    Reserved         = 2,  // BIOS/MMIO — do not touch
    AcpiReclaimable  = 3,  // ACPI tables — can reclaim after ACPI init
    AcpiNvs          = 4,  // ACPI non-volatile storage
    BadMemory        = 5,  // Reported bad by firmware
}

// Maximum number of tape regions we support from Multiboot2
const MAX_REGIONS: usize = 32;

static mut TAPE_REGIONS: [TapeRegion; MAX_REGIONS] = [TapeRegion {
    base: 0, length: 0, kind: TapeRegionKind::Reserved
}; MAX_REGIONS];
static mut REGION_COUNT: usize = 0;

/// Parse the Multiboot2 information structure to find memory map.
///
/// # Safety
/// `multiboot_info_phys` must be the physical address passed by GRUB in EBX.
/// Call this only once, before paging (we have no paging, so always safe here).
pub unsafe fn parse_memory_map(multiboot_info_phys: u32) {
    // Multiboot2 info structure layout:
    //   u32: total_size
    //   u32: reserved
    //   followed by tags...
    //
    // Each tag:
    //   u32: type
    //   u32: size
    //   u8[]: data (size - 8 bytes)
    //
    // Memory map tag type = 6
    // Memory map entry: u64 base_addr, u64 length, u32 type, u32 reserved

    // Since we have no paging, physical == virtual
    let info_base = multiboot_info_phys as *const u32;
    let total_size = *info_base;

    let mut offset: u32 = 8;  // Skip total_size and reserved fields

    while offset < total_size {
        let tag_ptr = (multiboot_info_phys + offset) as *const u32;
        let tag_type = *tag_ptr;
        let tag_size = *tag_ptr.add(1);

        if tag_type == 0 {
            break;  // End tag
        }

        if tag_type == 6 {
            // Memory map tag
            // Bytes 8..11: entry_size, bytes 12..15: entry_version
            let entry_size  = *tag_ptr.add(2);
            // let entry_version = *tag_ptr.add(3);  // unused

            let mut entry_offset: u32 = 16;  // Start of first entry (after tag header + entry_size + version)
            while entry_offset < tag_size {
                let entry_ptr = (multiboot_info_phys + offset + entry_offset) as *const u32;
                // Each entry: u64 base_addr, u64 length, u32 type, u32 reserved
                let base_low   = *entry_ptr;
                let base_high  = *entry_ptr.add(1);
                let len_low    = *entry_ptr.add(2);
                let _len_high  = *entry_ptr.add(3);
                let kind_raw   = *entry_ptr.add(4);

                // We are 32-bit — ignore any region above 4GB
                if base_high == 0 && REGION_COUNT < MAX_REGIONS {
                    let kind = match kind_raw {
                        1 => TapeRegionKind::Available,
                        3 => TapeRegionKind::AcpiReclaimable,
                        4 => TapeRegionKind::AcpiNvs,
                        5 => TapeRegionKind::BadMemory,
                        _ => TapeRegionKind::Reserved,
                    };
                    TAPE_REGIONS[REGION_COUNT] = TapeRegion {
                        base:   base_low,
                        length: len_low,
                        kind,
                    };
                    REGION_COUNT += 1;
                }

                entry_offset += entry_size;
            }
        }

        // Advance to next tag (8-byte aligned)
        let aligned_size = (tag_size + 7) & !7;
        offset += aligned_size;
    }

    crate::serial_println!("HEAD: found {} tape regions from Multiboot2", REGION_COUNT);
}

/// Return the parsed tape regions. TAPE domain calls this during init.
pub fn get_tape_memory_map() -> &'static [TapeRegion] {
    // SAFETY: parse_memory_map was called once during boot, before any concurrency.
    unsafe { &TAPE_REGIONS[..REGION_COUNT] }
}

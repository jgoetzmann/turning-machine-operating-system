// kernel/src/device/ata.rs
// ATA PIO disk driver — read/write 512-byte sectors
// Agent: DEVICE (Agent-06)

const ATA_DATA:    u16 = 0x1F0;
const ATA_SECTORS: u16 = 0x1F2;
const ATA_LBA_LO:  u16 = 0x1F3;
const ATA_LBA_MID: u16 = 0x1F4;
const ATA_LBA_HI:  u16 = 0x1F5;
const ATA_DRIVE:   u16 = 0x1F6;
const ATA_CMD:     u16 = 0x1F7;  // write = command, read = status
const ATA_STATUS:  u16 = 0x1F7;

const ATA_CMD_READ:  u8 = 0x20;
const ATA_CMD_WRITE: u8 = 0x30;
const ATA_STATUS_BSY: u8 = 0x80;
const ATA_STATUS_DRQ: u8 = 0x08;
const ATA_STATUS_ERR: u8 = 0x01;

#[derive(Debug)]
pub enum AtaError { Timeout, Error, NoDisk }

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack));
}
unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port, options(nomem, nostack));
    v
}
unsafe fn inw(port: u16) -> u16 {
    let v: u16;
    core::arch::asm!("in ax, dx", out("ax") v, in("dx") port, options(nomem, nostack));
    v
}
unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack));
}

static mut DISK_PRESENT: bool = false;

/// Poll ATA status until BSY clears. Returns Err on timeout or error.
unsafe fn ata_poll() -> Result<(), AtaError> {
    let mut timeout = 100_000u32;
    loop {
        let status = inb(ATA_STATUS);
        if status == 0xFF { return Err(AtaError::NoDisk); }
        if status & ATA_STATUS_ERR != 0 { return Err(AtaError::Error); }
        if status & ATA_STATUS_BSY == 0 && status & ATA_STATUS_DRQ != 0 { return Ok(()); }
        timeout -= 1;
        if timeout == 0 { return Err(AtaError::Timeout); }
    }
}

/// Detect ATA primary master drive. Returns false if no disk.
pub fn ata_init() -> bool {
    unsafe {
        if inb(ATA_STATUS) == 0xFF {
            DISK_PRESENT = false;
            return false;
        }
        DISK_PRESENT = true;
        true
    }
}

/// Read one 512-byte sector at LBA `lba` into `buf`.
pub fn ata_read_sector(lba: u32, buf: &mut [u8; 512]) -> Result<(), AtaError> {
    unsafe {
        if !DISK_PRESENT { return Err(AtaError::NoDisk); }
        // LBA28 addressing: drive 0, LBA mode
        outb(ATA_DRIVE,   0xE0 | ((lba >> 24) as u8 & 0x0F));
        outb(ATA_SECTORS, 1);
        outb(ATA_LBA_LO,  (lba & 0xFF) as u8);
        outb(ATA_LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(ATA_LBA_HI,  ((lba >> 16) & 0xFF) as u8);
        outb(ATA_CMD,     ATA_CMD_READ);

        ata_poll()?;

        // Read 256 words (512 bytes)
        for i in (0..512).step_by(2) {
            let word = inw(ATA_DATA);
            buf[i]   = (word & 0xFF) as u8;
            buf[i+1] = (word >> 8)   as u8;
        }
        Ok(())
    }
}

/// Write one 512-byte sector at LBA `lba` from `buf`.
pub fn ata_write_sector(lba: u32, buf: &[u8; 512]) -> Result<(), AtaError> {
    unsafe {
        if !DISK_PRESENT { return Err(AtaError::NoDisk); }
        outb(ATA_DRIVE,   0xE0 | ((lba >> 24) as u8 & 0x0F));
        outb(ATA_SECTORS, 1);
        outb(ATA_LBA_LO,  (lba & 0xFF) as u8);
        outb(ATA_LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(ATA_LBA_HI,  ((lba >> 16) & 0xFF) as u8);
        outb(ATA_CMD,     ATA_CMD_WRITE);

        ata_poll()?;

        for i in (0..512).step_by(2) {
            let word = buf[i] as u16 | (buf[i+1] as u16) << 8;
            outw(ATA_DATA, word);
        }
        Ok(())
    }
}

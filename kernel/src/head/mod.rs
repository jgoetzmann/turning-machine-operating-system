// kernel/src/head/mod.rs
// HEAD domain — Boot & CPU Initialization
// Agent: Agent-02
// TM Role: Position the tape head. Set up the machine so it can begin reading.
//
// Read docs/agents/HEAD.md before modifying this file.

pub mod serial;   // COM1 UART — first thing that works at boot
pub mod gdt;      // Flat 32-bit GDT
pub mod idt;      // IDT structure (TRANS domain fills the actual gates)
pub mod memmap;   // Parse Multiboot2 memory map → TapeRegion list

// Re-export the TapeRegion type so TAPE domain can use it
pub use memmap::TapeRegion;

/// Entry point called from _start (boot.asm) after minimal ASM setup.
///
/// At this point:
/// - CPU is in 32-bit protected mode (GRUB set this up)
/// - Stack is set to KERNEL_STACK_TOP (boot.asm did this)
/// - BSS is zeroed (boot.asm did this)
/// - Multiboot2 magic and info pointer are on the stack
///
/// This function must NOT return.
///
/// # Safety
/// Called from assembly. Multiboot2 guarantees are in effect.
/// `multiboot_info_phys` is a physical address valid at call time.
#[no_mangle]
pub unsafe extern "C" fn boot_init(multiboot_magic: u32, multiboot_info_phys: u32) -> ! {
    // Step 1: COM1 serial — earliest possible debug output
    serial::serial_init();
    serial_println!("HEAD: Turing Machine OS booting...");
    serial_println!("HEAD: target = i686 (32-bit protected mode, no paging)");

    // Step 2: Validate Multiboot2 magic
    const MULTIBOOT2_MAGIC: u32 = 0x36d76289;
    if multiboot_magic != MULTIBOOT2_MAGIC {
        serial_println!("HEAD: FATAL — bad multiboot2 magic: {:#010x}", multiboot_magic);
        serial_println!("HEAD: expected: {:#010x}", MULTIBOOT2_MAGIC);
        loop { core::arch::asm!("hlt") }
    }
    serial_println!("HEAD: multiboot2 magic OK ({:#010x})", multiboot_magic);

    // Step 3: Load our own flat GDT
    // (GRUB's GDT may not have user segments or TSS — set up ours)
    gdt::gdt_init();
    gdt::gdt_load();
    serial_println!("HEAD: GDT loaded (flat 32-bit segments)");

    // Step 4: Initialize empty IDT
    // (TRANS domain fills the actual gate entries during ctrl::machine_boot)
    idt::idt_init();
    serial_println!("HEAD: IDT initialized (gates empty — TRANS will fill them)");

    // Step 5: Parse Multiboot2 memory map
    // (Saves tape regions for TAPE domain to use)
    memmap::parse_memory_map(multiboot_info_phys);
    serial_println!("HEAD: tape memory map parsed ({} regions)",
                     memmap::get_tape_memory_map().len());

    serial_println!("HEAD: boot complete — handing off to CTRL");

    // Step 6: Hand off to the state machine — never returns
    crate::ctrl::machine_boot()
}

/// Physical tape regions discovered from Multiboot2 memory map.
/// TAPE domain calls this to initialize its allocator.
pub fn get_tape_memory_map() -> &'static [TapeRegion] {
    memmap::get_tape_memory_map()
}

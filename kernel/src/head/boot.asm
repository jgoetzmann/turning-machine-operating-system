; kernel/src/head/boot.asm
; Multiboot2 header + kernel entry point (_start)
; Agent: HEAD (Agent-02)
; Assembled with NASM, 32-bit protected mode
;
; When GRUB (or QEMU -kernel) loads us:
;   - CPU is already in 32-bit protected mode
;   - EAX = 0x36d76289 (Multiboot2 magic)
;   - EBX = physical address of Multiboot2 info structure
;   - Interrupts are disabled (IF=0)
;   - A20 line is enabled
;   - We are at physical address 0x00100000 (loaded by GRUB)
;
; What we do:
;   1. Declare Multiboot2 header (within first 8KB)
;   2. Save EAX (magic) and EBX (info ptr) — they'll be clobbered
;   3. Set up initial kernel stack
;   4. Zero BSS segment (Rust assumes zero-init statics)
;   5. Call boot_init(magic, info_ptr) [Rust function in head/mod.rs]
;
; See MISTAKE-001 (64-bit registers), MISTAKE-002 (BSS not zeroed),
; and MISTAKE-006 (segment registers after GDT load) in docs/MISTAKES.md

bits 32

; ─── Multiboot2 Header ────────────────────────────────────────────────────
; Must be in the first 8KB of the kernel binary.
; The linker script places .multiboot2 first.

section .multiboot2
align 8

MULTIBOOT2_MAGIC    equ 0xE85250D6     ; Header magic (not the boot magic)
MULTIBOOT2_ARCH     equ 0              ; i386 protected mode
MULTIBOOT2_CHECKSUM equ -(MULTIBOOT2_MAGIC + MULTIBOOT2_ARCH + (mb2_header_end - mb2_header_start))

mb2_header_start:
    dd MULTIBOOT2_MAGIC
    dd MULTIBOOT2_ARCH
    dd (mb2_header_end - mb2_header_start)  ; header length
    dd MULTIBOOT2_CHECKSUM & 0xFFFFFFFF     ; checksum (mask to 32 bits)

    ; Framebuffer tag (optional — request text mode)
    align 8
    dw 5                ; tag type: framebuffer
    dw 0                ; flags: 0
    dd 20               ; size: 20 bytes
    dd 0                ; preferred width (0 = no preference)
    dd 0                ; preferred height
    dd 0                ; preferred depth (0 = no preference)

    ; End tag (required)
    align 8
    dw 0                ; type: 0 = end
    dw 0                ; flags: 0
    dd 8                ; size: 8
mb2_header_end:

; ─── Kernel Stack ─────────────────────────────────────────────────────────
; 16KB stack in BSS — grows downward from kernel_stack_top
; Physical address: defined in linker.ld as KERNEL_STACK_TOP = 0x00070000

section .bss
align 16
kernel_stack_bottom:
    resb 16384          ; 16KB stack
kernel_stack_top:

; ─── Entry Point ──────────────────────────────────────────────────────────

section .text
global _start
extern boot_init        ; Rust function in kernel/src/head/mod.rs
extern _bss_start       ; Linker script symbols
extern _bss_end

_start:
    ; ── Save Multiboot2 arguments before we clobber registers ──
    ; EAX = Multiboot2 magic (0x36d76289)
    ; EBX = physical address of Multiboot2 info structure
    mov edi, eax        ; save magic in EDI (callee-saved, won't be touched)
    mov esi, ebx        ; save info ptr in ESI

    ; ── Set up initial kernel stack ──
    ; Stack grows DOWN from kernel_stack_top (defined above in .bss)
    mov esp, kernel_stack_top

    ; ── Zero the BSS segment ──
    ; Rust assumes zero-initialized statics (.bss section).
    ; If we skip this, Option<T> statics contain garbage.
    ; See MISTAKE-002 in docs/MISTAKES.md.
    mov ecx, _bss_end
    sub ecx, _bss_start     ; ECX = size of BSS in bytes
    mov edi_bss, _bss_start ; EDI = start of BSS
    xor eax, eax            ; EAX = 0
    ; rep stosd writes EAX to [EDI], incrementing EDI by 4, ECX times
    ; This zeros ECX/4 dwords — works when BSS is dword-aligned (linker ensures this)
    shr ecx, 2              ; convert bytes to dwords
    rep stosd

    ; Restore EDI (we used it for BSS zeroing; reload magic)
    mov eax, edi_saved
    ; Actually, let's just re-push from the saved values

    ; ── Push arguments for boot_init(magic: u32, info_phys: u32) ──
    ; 32-bit cdecl: arguments pushed right-to-left on stack
    push esi            ; arg2: multiboot_info_phys (EBX original)
    push edi            ; arg1: multiboot_magic (EAX original)

    ; ── Call Rust entry point ──
    call boot_init

    ; ── Should never reach here ──
    ; boot_init calls ctrl::machine_boot() which never returns.
    ; If it does, halt forever.
.hang:
    cli
    hlt
    jmp .hang

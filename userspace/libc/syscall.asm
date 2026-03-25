; userspace/libc/syscall.asm
; 32-bit int 0x80 system call wrappers for userspace
; Assembled with NASM, 32-bit, for i686-elf-gcc
; Agent: SHELL (Agent-10)
;
; Calling convention: cdecl (caller pushes args right-to-left, caller cleans)
; int 0x80 ABI: eax=num, ebx=arg1, ecx=arg2, edx=arg3
; Return value: eax (negative = errno)

bits 32

section .text

; long sys_write(int fd, const void *buf, long len)
global sys_write
sys_write:
    push ebx
    mov  eax, 4          ; SYS_WRITE = 4
    mov  ebx, [esp+8]    ; fd
    mov  ecx, [esp+12]   ; buf
    mov  edx, [esp+16]   ; len
    int  0x80
    pop  ebx
    ret

; void sys_exit(int code) __attribute__((noreturn))
global sys_exit
sys_exit:
    mov  eax, 1          ; SYS_EXIT = 1
    mov  ebx, [esp+4]    ; exit code
    int  0x80
    ; should not return — loop forever if it does
.hang:
    jmp  .hang

; long sys_getpid(void)
global sys_getpid
sys_getpid:
    mov  eax, 20         ; SYS_GETPID = 20
    xor  ebx, ebx
    int  0x80
    ret

; void sys_yield(void)
global sys_yield
sys_yield:
    mov  eax, 158        ; SYS_YIELD = sched_yield = 158
    xor  ebx, ebx
    int  0x80
    ret

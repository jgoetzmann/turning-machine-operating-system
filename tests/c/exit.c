// tests/c/exit.c
// Minimal freestanding i386 program that just exits via int 0x80.

static inline void sys_exit(int code) {
    asm volatile(
        "int $0x80"
        :
        : "a"(1), "b"(code)
        : "memory"
    );
    for (;;) { }
}

void _start(void) {
    sys_exit(42);
}


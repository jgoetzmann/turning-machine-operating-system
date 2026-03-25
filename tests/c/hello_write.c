// tests/c/hello_write.c
// Freestanding i386 ELF intended for TMOS-style int 0x80 ABI experiments.
// Host-side smoke test: this file should compile to ELF32 i386 without libc.

typedef unsigned int u32;

static inline int sys_write(int fd, const char *buf, u32 len) {
    int ret;
    asm volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(4), "b"(fd), "c"(buf), "d"(len)
        : "memory"
    );
    return ret;
}

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
    const char msg[] = "hello from C via int 0x80\n";
    (void)sys_write(1, msg, (u32)(sizeof(msg) - 1));
    sys_exit(0);
}


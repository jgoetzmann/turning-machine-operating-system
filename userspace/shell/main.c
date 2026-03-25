/* userspace/shell/main.c
 * Turing Machine OS — minimal shell
 * Agent: SHELL (Agent-10)
 *
 * Build with: i686-elf-gcc -m32 -nostdlib -nostdinc -static -o SHELL.ELF main.c
 * Uses int 0x80 syscall ABI via libc/syscall.asm
 */

/* syscall prototypes — implemented in libc/syscall.asm */
long sys_write(int fd, const void *buf, long len);
void sys_exit(int code) __attribute__((noreturn));
long sys_getpid(void);
void sys_yield(void);

/* ── Minimal string helpers (no libc) ────────────────────────────────── */
static long tm_strlen(const char *s) {
    long n = 0;
    while (s[n]) n++;
    return n;
}

static void tm_write(const char *s) {
    sys_write(1, s, tm_strlen(s));
}

static int tm_strcmp(const char *a, const char *b) {
    while (*a && *a == *b) { a++; b++; }
    return (unsigned char)*a - (unsigned char)*b;
}

/* ── Read a line from keyboard ────────────────────────────────────────── */
/* TODO (Agent-10): implement keyboard read via sys_read once IPC is wired */
static long tm_readline(char *buf, long max) {
    /* Placeholder: in QEMU test mode, the kernel injects the test command */
    /* Real implementation reads chars via sys_read(0, &ch, 1) in a loop  */
    (void)buf; (void)max;
    return 0;
}

/* ── Built-in: echo ───────────────────────────────────────────────────── */
static void builtin_echo(char *line, long start) {
    tm_write(line + start);
    tm_write("\n");
}

/* ── Built-in: help ───────────────────────────────────────────────────── */
static void builtin_help(void) {
    tm_write("Turing Machine OS Shell\n");
    tm_write("Commands: echo <text>, help, exit\n");
    tm_write("This shell runs on the process tape at ");
    /* Print PID as proof the syscall works */
    char pid_buf[12];
    long pid = sys_getpid();
    long i = 10;
    pid_buf[11] = '\0';
    pid_buf[i] = '\0';
    if (pid == 0) { pid_buf[--i] = '0'; }
    while (pid > 0) { pid_buf[--i] = '0' + (pid % 10); pid /= 10; }
    tm_write(pid_buf + i);
    tm_write("\n");
}

/* ── Main shell loop ──────────────────────────────────────────────────── */
void _start(void) {
    tm_write("TM Shell starting...\n");
    tm_write("Type 'help' for available commands.\n\n");

    char line[256];

    while (1) {
        tm_write("TM> ");

        long len = tm_readline(line, sizeof(line));
        if (len <= 0) {
            /* No input — yield and try again */
            sys_yield();
            continue;
        }

        /* Null-terminate and strip trailing newline */
        line[len] = '\0';
        if (len > 0 && line[len-1] == '\n') line[--len] = '\0';

        if (len == 0) continue;

        /* Parse command */
        if (tm_strcmp(line, "exit") == 0) {
            tm_write("Goodbye.\n");
            sys_exit(0);
        }
        else if (tm_strcmp(line, "help") == 0) {
            builtin_help();
        }
        else if (len >= 5 && line[0]=='e' && line[1]=='c' && line[2]=='h' &&
                 line[3]=='o' && line[4]==' ') {
            builtin_echo(line, 5);
        }
        else {
            tm_write("Unknown command: ");
            tm_write(line);
            tm_write("\nType 'help' for available commands.\n");
        }
    }
}

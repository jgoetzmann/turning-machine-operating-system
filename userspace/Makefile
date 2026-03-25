# userspace/Makefile
# Build userspace programs for i686-elf target
# Requires: i686-elf-gcc cross-compiler

CC      = i686-elf-gcc
AS      = nasm
CFLAGS  = -m32 -nostdlib -nostdinc -static -O2 -ffreestanding \
          -fno-stack-protector -fno-pie -no-pie
ASFLAGS = -f elf32

LIBC_OBJS = libc/syscall.o
SHELL_ELF = shell/SHELL.ELF

.PHONY: all clean

all: $(SHELL_ELF)
	@echo "✅ Userspace built: $(SHELL_ELF)"

libc/syscall.o: libc/syscall.asm
	$(AS) $(ASFLAGS) -o $@ $<

shell/SHELL.ELF: shell/main.c $(LIBC_OBJS)
	$(CC) $(CFLAGS) -o $@ $^
	@file $@ | grep -q "32-bit" && echo "✅ SHELL.ELF is 32-bit" || echo "❌ Wrong arch!"

clean:
	rm -f libc/*.o shell/*.ELF

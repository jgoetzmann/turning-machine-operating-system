# Makefile — Turing Machine OS
# Owned by: BUILD domain (Agent-01)
# Target: i686-unknown-none (32-bit bare metal)

.PHONY: all build build-debug qemu qemu-headless qemu-test qemu-test-all \
        build-userspace make-disk clean clean-locks venv \
        dispatch-status snapshot fmt clippy check help

KERNEL_ELF     := target/i686-unknown-none/release/kernel
KERNEL_ELF_DBG := target/i686-unknown-none/debug/kernel
QEMU           := qemu-system-i386
FAT12_IMG      := tests/fixtures/fat12.img
PYTHON         := .venv/bin/python3

# ─── Default ─────────────────────────────────────────────────────────────────

all: build

# ─── Build ───────────────────────────────────────────────────────────────────

build:
	cargo build --release -p kernel
	@echo "✅ Build OK (32-bit): $(KERNEL_ELF)"
	@file $(KERNEL_ELF) | grep -q "32-bit" && echo "✅ ELF is 32-bit" || echo "❌ WARNING: ELF may not be 32-bit!"

build-debug:
	cargo build -p kernel
	@echo "✅ Debug build: $(KERNEL_ELF_DBG)"

build-userspace:
	@command -v i686-elf-gcc >/dev/null 2>&1 || { echo "ERROR: i686-elf-gcc not found. Install i686-elf cross-compiler."; exit 1; }
	$(MAKE) -C userspace
	@echo "✅ Userspace built"

# ─── QEMU ────────────────────────────────────────────────────────────────────
# IMPORTANT: Always qemu-system-i386 (32-bit), NEVER qemu-system-x86_64

qemu: build
	$(QEMU) \
		-kernel $(KERNEL_ELF) \
		-serial stdio \
		-m 128M \
		-display gtk \
		-drive file=$(FAT12_IMG),format=raw,if=ide \
		-no-reboot

qemu-headless: build
	$(QEMU) \
		-kernel $(KERNEL_ELF) \
		-serial stdio \
		-m 128M \
		-display none \
		-drive file=$(FAT12_IMG),format=raw,if=ide \
		-no-reboot

qemu-display: build
	$(QEMU) \
		-kernel $(KERNEL_ELF) \
		-serial file:serial.log \
		-m 128M \
		-display gtk \
		-vga std \
		-drive file=$(FAT12_IMG),format=raw,if=ide \
		-no-reboot

# Run a specific test
# Usage: make qemu-test TEST=head_boot_basic
# Usage: make qemu-test TEST=store_fat12 TIMEOUT=20
qemu-test: build
	@if [ -z "$(TEST)" ]; then \
		echo "Usage: make qemu-test TEST=test_name [TIMEOUT=30]"; \
		exit 1; \
	fi
	$(PYTHON) tools/test_runner.py --test $(TEST) --timeout $(or $(TIMEOUT),30)

qemu-test-all: build
	$(PYTHON) tools/test_runner.py --all --timeout 30

test-list:
	$(PYTHON) tools/test_runner.py --list

# ─── Disk Images ──────────────────────────────────────────────────────────────

make-disk:
	@echo "Creating FAT12 test disk image..."
	bash tools/make_fat12.sh
	@echo "✅ $(FAT12_IMG) created"

# ─── Code Quality ─────────────────────────────────────────────────────────────

fmt:
	cargo fmt --all

clippy:
	cargo clippy --target i686-unknown-none -- -D warnings

check: fmt clippy
	@echo "✅ Format and lint OK"

# Verify the ELF is actually 32-bit (run after every build)
verify-32bit: build
	@echo "Checking ELF architecture..."
	@file $(KERNEL_ELF)
	@file $(KERNEL_ELF) | grep -q "Intel 80386" && \
		echo "✅ CONFIRMED: 32-bit i386 ELF" || \
		(echo "❌ WRONG ARCH: Expected Intel 80386" && exit 1)

# ─── Python Tooling ───────────────────────────────────────────────────────────

venv:
	bash tools/setup_venv.sh

dispatch-status: venv
	$(PYTHON) tools/dispatch_generator.py --status

# Usage: make dispatch-create DOMAIN=CTRL TITLE="Implement machine loop"
dispatch-create:
	@if [ -z "$(DOMAIN)" ] || [ -z "$(TITLE)" ]; then \
		echo "Usage: make dispatch-create DOMAIN=CTRL TITLE='task title'"; \
		exit 1; \
	fi
	$(PYTHON) tools/dispatch_generator.py --create --domain $(DOMAIN) --title "$(TITLE)"

# Usage: make snapshot DOMAIN=CTRL VERSION=2
snapshot:
	@if [ -z "$(DOMAIN)" ] || [ -z "$(VERSION)" ]; then \
		echo "Usage: make snapshot DOMAIN=CTRL VERSION=2"; \
		exit 1; \
	fi
	$(PYTHON) tools/context_compressor.py --domain $(DOMAIN) --version $(VERSION)

# ─── Clean ────────────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf tests/results/ serial.log
	@echo "✅ Clean"

clean-locks:
	@echo "Clearing stale task locks (confirm all are stale first)..."
	rm -f task-locks/*.lock
	@echo "✅ Locks cleared"

# ─── Help ─────────────────────────────────────────────────────────────────────

help:
	@echo ""
	@echo "Turing Machine OS — Make Targets"
	@echo "================================="
	@echo "NOTE: This project is 32-bit i686. NEVER use qemu-system-x86_64."
	@echo ""
	@echo "Build:"
	@echo "  make build              — 32-bit kernel ELF (release)"
	@echo "  make build-debug        — debug build"
	@echo "  make build-userspace    — userspace + shell (needs i686-elf-gcc)"
	@echo "  make verify-32bit       — confirm output is Intel 80386 ELF"
	@echo "  make clean              — clean build artifacts"
	@echo ""
	@echo "QEMU (always i386, never x86_64):"
	@echo "  make qemu               — interactive (serial + display)"
	@echo "  make qemu-headless      — serial only"
	@echo "  make qemu-display       — VGA display"
	@echo "  make qemu-test TEST=n   — run a specific test"
	@echo "  make qemu-test-all      — run all tests"
	@echo "  make test-list          — list all tests"
	@echo ""
	@echo "Disk:"
	@echo "  make make-disk          — create FAT12 test disk image"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt                — format"
	@echo "  make clippy             — lint"
	@echo "  make check              — fmt + clippy"
	@echo ""
	@echo "Agent Tooling:"
	@echo "  make venv               — set up Python venv"
	@echo "  make dispatch-status    — show task queue"
	@echo "  make dispatch-create DOMAIN=X TITLE='...' — new card"
	@echo "  make snapshot DOMAIN=X VERSION=N — context snapshot"
	@echo "  make clean-locks        — remove stale task locks"
	@echo ""

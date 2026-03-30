# TuringOS — Remember

> **APPEND ONLY. Never delete or overwrite entries.**  
> This file is the institutional memory of the project.  
> Format for new entries:
> ```
> ## [YYYY-MM-DD] <component> — <short title>
> <body>
> ```

---

## [PROJECT INIT] Architecture — TM model is a hard constraint

The Turing Machine mapping is not a metaphor or a design pattern. It is a hard constraint.

- Tape = `mem[65536]`. No exceptions. No allocations outside this array from inside the emulator.
- Head = 8080 PC. The only way to "move the head" is to execute 8080 instructions.
- States = the `kernel_state_t` enum. The kernel FSM is the finite state control.
- Transition function = `cpu_step()` + `bios_dispatch()`. These are the ONLY legal state-transition sites.

Any architecture that violates this mapping must be rejected, even if it would be simpler or faster.

---

## [PROJECT INIT] Architecture — Why Intel 8080, not Z80

Z80 is an extension of the 8080 that adds additional instructions (IX/IY registers, DJNZ, etc.). Targeting Z80 would require implementing a larger, more complex instruction set. The 8080 is the minimal complete 8-bit CPU that CP/M ran on historically, and it keeps the emulator scope tight. If a Z80-specific instruction appears in any test program or shell source, it must be replaced with 8080 equivalents.

---

## [PROJECT INIT] Architecture — `.com` file format

`.com` files are raw flat binaries. No headers. No relocation tables. No segments. The OS loads the entire file into memory starting at address `0x0100`. The entry point is always byte 0 of the file (address `0x0100` in memory). The compiler must generate position-dependent code assuming base address `0x0100`.

---

## [PROJECT INIT] Architecture — No malloc in kernel or emulator

The kernel and emulator code must use zero heap allocation (`malloc`, `calloc`, `realloc` are forbidden in `src/`). All data structures are statically declared. This is not a performance optimization — it is a correctness requirement. The "memory" of this system is `mem[65536]`, and allocating host memory for kernel data structures would break the TM model.

The only exception is the compiler (`src/compiler/`), which runs on the host and may use `malloc` for its AST nodes during compilation. The compiler is a host tool, not part of the emulated system.

---

## [PROJECT INIT] Architecture — Filesystem buffer cache in `mem[]`

The filesystem buffer cache is located at `mem[0xC000..0xEFFF]` (12KB). This means disk sectors read by the BIOS are written directly into the tape. This is intentional — it makes disk I/O visible on the tape visualization and keeps the TM model clean. The FS code must never use a separate host-side buffer for sector data.

---

## [PROJECT INIT] Architecture — Tape snapshot frequency

The kernel writes the tape snapshot to `/tmp/turingos_tape.bin` every 1000 ticks. This is a tunable constant (`#define TAPE_SNAP_INTERVAL 1000` in `kernel.h`). If the visualizer feels sluggish or the OS feels slow due to file I/O, reduce this to 5000 or increase to 100. The default of 1000 is a starting point, not a guarantee.

---

## [PROJECT INIT] Known pitfalls — 8080 flag computation

The 8080 flag register has several non-obvious behaviors that are frequently wrong in emulator implementations:

1. **Auxiliary Carry (AC)**: Set when there is a carry out of bit 3 into bit 4. Most ops set this; it is used by `DAA`. Easy to forget.
2. **Parity (P)**: Set when the number of set bits in the result is even. Must be computed for all arithmetic/logic ops.
3. **DAA**: The Decimal Adjust Accumulator instruction is complex. It adjusts A after BCD arithmetic using the AC and CY flags. Implement this carefully and test it explicitly.
4. **PUSH PSW / POP PSW**: Pushes/pops the A register and flags register together as a 16-bit pair. The flags are packed in a specific bit layout: `S Z 0 AC 0 P 1 CY`. Bit 1 is always 1, bits 3 and 5 are always 0. Get this wrong and the stack will corrupt A.
5. **CMP**: Does not modify the accumulator. It performs a subtraction and sets flags but discards the result. Easy to accidentally write the result back.

These are all well-documented in the Intel 8080 Programmer's Manual. The test suite must cover each of these edge cases.

---

## [PROJECT INIT] Known pitfalls — CP/M BIOS calling convention

This implementation uses `OUT 0x01` as the BIOS call trigger (port 0x01). The function ID is in register A. This differs slightly from standard CP/M which uses a jump table at 0x0000. Both approaches are used in real CP/M implementations. We use the `OUT` port approach because it's cleaner for the emulator to intercept than scanning jumps. The shell and compiler must use this convention. Document any deviation here.

---

## [PROJECT INIT] Known pitfalls — pygame shared memory on macOS

On macOS, `/tmp` is actually `/private/tmp`. The mmap path in `tape_bridge.py` should use `os.path.realpath('/tmp/turingos_tape.bin')` to resolve the symlink. Docker avoids this issue entirely. If the visualizer silently fails to find the tape file on macOS, this is likely the cause.

---

## [PROJECT INIT] Known pitfalls — raw terminal mode

The BIOS CONIN function must put the host terminal into raw mode (no echo, no line buffering) so the shell can read individual keystrokes. On POSIX this means using `tcsetattr` with `ICANON` and `ECHO` cleared. The kernel must restore the terminal to cooked mode on `KS_HALT` or on any unhandled signal (set up a `SIGINT` handler that restores terminal state before exit). Failure to restore terminal state leaves the user's terminal broken after the OS exits.

---

## [PROJECT INIT] Decision — C99 not C11

The codebase uses C99. C11 added `_Alignas`, `_Atomic`, `_Generic`, `_Static_assert`, and other features we don't need. Using C99 keeps the code more portable and avoids any temptation to use VLAs (which are technically optional in C11) or threads. The compiler flag must be `-std=c99`.

---

## [PROJECT INIT] Decision — Docker is primary, venv is visualizer-only

The full OS build and test suite runs in Docker. This ensures reproducibility. The Python virtualenv in `viz/.venv` is provided as a convenience for running the visualizer without Docker, but it is not the primary development path. Agents working on the visualizer may use either. Agents working on the kernel/emulator must use Docker or a system with gcc + make installed.

---

_Add new entries below this line._
## [2026-03-30] build/disk — Initial scaffold and bootstrap build
Bootstrapped the repository from docs-only state into a compilable C99 project skeleton.
Added `Makefile`, `Dockerfile`, `docker-compose.yml`, core directory layout, C stub modules, and `tools/mkdisk.c`.
Validated with `make all`, `make run`, and `make disk`; `disk.img` is generated at 512,512 bytes with directory bytes initialized to `0xE5`.
## [2026-03-30] emu/mem — Boundary tests wired into make test
Added `tests/emu/test_mem.c` to validate zero-init and read/write round-trips at boundary addresses (`0x0000`, `0x7FFF`, `0xFFFF`) and to verify `mem_raw()` exposes the same backing tape bytes.
Replaced placeholder `make test` behavior with `tests/run_tests.sh` so Phase 1 memory verification runs as part of the normal build flow.
## [2026-03-30] emu/cpu — Data-transfer decoder foundation
Implemented fetch/decode in `cpu_step()` with working data-transfer opcodes: `MOV`, `MVI`, `LXI`, `LDA`, `STA`, `LHLD`, `SHLD`, `XCHG`, `XTHL`, `SPHL`, `PCHL`; also wired `NOP` and `HLT` handling in the same decoder pass.
Added `tests/emu/test_cpu_data_transfer.c` and integrated it into `tests/run_tests.sh` to validate core register/memory transfer paths and `HLT` behavior while the remaining opcode groups are still pending.
## [2026-03-30] emu/cpu — Arithmetic opcodes and baseline flag behavior
Implemented arithmetic opcode coverage in `cpu_step()` for `ADD/ADI/ADC`, `SUB/SUI/SBB/SBI`, `INR/DCR`, `INX/DCX`, `DAD`, and `DAA`, including baseline updates for `S`, `Z`, `P`, `AC`, and `CY` where those instructions define flag changes.
Added `tests/emu/test_cpu_arithmetic.c` and wired it into `tests/run_tests.sh` so arithmetic register effects, carry/zero behavior, 16-bit increments/decrements, and a DAA sanity path are now regression-tested.
## [2026-03-30] emu/cpu — Logic and rotate opcodes
Implemented `ANA/ANI`, `ORA/ORI`, `XRA/XRI` with `CY` and `AC` cleared and `S/Z/P` from result; `CMP/CPI` set flags like `SUB` without changing `A`; `RLC/RRC/RAL/RAR` update `CY` only; `CMA` leaves flags; `STC`/`CMC` affect carry only.
Added `tests/emu/test_cpu_logic.c` and hooked it into `tests/run_tests.sh`.
## [2026-03-30] emu/cpu — Branch, call, and return opcodes
Implemented `JMP` and all conditional jump forms, `CALL` and all conditional call forms, and `RET` plus all conditional return forms. Added helper routines for 16-bit immediate fetch, stack push/pop for return addresses, and condition decoding from flags.
Added `tests/emu/test_cpu_branch.c` and wired it into `tests/run_tests.sh` to validate taken/not-taken branches, unconditional call/return behavior, and conditional call/return flow.
## [2026-03-30] emu/cpu — Stack PUSH/POP including PSW packing
Implemented `PUSH`/`POP` for `BC`, `DE`, `HL`, and `PSW`. For `PSW`, flag bytes are packed/unpacked as `S Z 0 AC 0 P 1 CY` so bit 1 stays set and bits 3/5 remain clear.
Added `tests/emu/test_cpu_stack.c` and integrated it into `tests/run_tests.sh` to validate stack pointer movement, memory byte order, and PSW round-trip behavior.
## [2026-03-30] emu/cpu — IN/OUT latch for BIOS dispatch
Implemented `IN` (`0xDB`) and `OUT` (`0xD3`). `IN` reads from a CPU-hosted 256-byte input port table (`io_in_ports`) into register `A`; `OUT` captures `A` into an output latch (`io_out_value`) with `io_out_port` and `io_out_pending` so kernel/BIOS code can observe and clear it.
Added `tests/emu/test_cpu_io.c` and integrated it into `tests/run_tests.sh` to verify IN reads, OUT latch values, and pending-flag behavior across successive writes.
## [2026-03-30] emu/cpu — Control opcodes RST/EI/DI/RIM/SIM
Implemented `RST n` as a call-like vector jump (push return address, jump to `n*8`), plus `EI`, `DI`, `RIM`, and `SIM`. Added small CPU state fields for interrupt enable and simple `rim_value`/`sim_value` latches to support deterministic tests and future BIOS/kernel integration.
Added `tests/emu/test_cpu_control.c` and integrated it into `tests/run_tests.sh` to validate interrupt-toggle behavior, RIM/SIM data flow, and RST stack/PC behavior.
## [2026-03-30] emu/cpu — Corrected AND auxiliary carry semantics
Adjusted flag logic so `ANA/ANI` set `AC=1` and clear `CY`, while `ORA/ORI` and `XRA/XRI` keep both `AC` and `CY` cleared. This aligns with Intel 8080 flag behavior and closes a subtle mismatch in earlier logic handling.
Expanded `tests/emu/test_cpu_logic.c` to assert `AC/CY` expectations for `ANI`, `XRI`, and `ORI`.
## [2026-03-30] tests/emu — Opcode group smoke harness
Added `tests/emu/test_opcodes.c` as a compact opcode-by-group verification pass that exercises representative instructions across all implemented groups (data transfer, arithmetic, logic, branch, stack, I/O, control) and confirms core register/PC/stack effects.
Integrated the new test into `tests/run_tests.sh` so every `make test` run now includes the opcode-group smoke suite.
## [2026-03-30] tests/emu — Dedicated HLT and boundary flag coverage
Added `tests/emu/test_cpu_hlt.c` to assert explicit `HLT` semantics: `cpu->halted` is set, `PC` advances by one for the `HLT` fetch, and subsequent `cpu_step()` calls are no-ops while halted.
Added `tests/emu/test_cpu_flags.c` to cover boundary flag behavior for `ADD`, `SUB`, `AND`, `OR`, and `XOR`, with explicit checks for `S`, `Z`, `AC`, `P`, and `CY` where applicable. Wired both tests into `tests/run_tests.sh`.
## [2026-03-30] bios — Dispatch table and output queue tests
Implemented BIOS dispatch handling for function IDs `0x01`–`0x0E`, with concrete behavior for `CONIN` (`A <- getchar()` or `0` on EOF) and `CONOUT` (enqueue register `C` into the BIOS output ring), while keeping remaining IDs as explicit stubs for later FS integration.
Kept `bios_pending_output()` / `bios_get_output()` as the queue drain interface and made dispatch clear `cpu->io_out_pending` after handling.
Added `tests/bios/test_bios.c` and integrated it into `tests/run_tests.sh` to verify queue init/drain behavior, repeated wraparound-safe enqueue/dequeue cycles, `CONIN` read path via `ungetc`, and pending-flag clearing.
## [2026-03-30] bios/fs — Disk parameter stubs and sector delegation wiring
Implemented BIOS disk-parameter state for `SELDISK`, `SETTRK`, `SETSEC`, and `SETDMA` with defaults reset by `bios_init()`, then used these values in `READ`/`WRITE` dispatch handling.
Wired BIOS `READ` (`0x0D`) and `WRITE` (`0x0E`) to `fs_read_sector` / `fs_write_sector` hooks and return status in `A` (`0` success, `1` failure), including a DMA bounds guard for 256-byte sector transfers.
Added `fs_read_sector` / `fs_write_sector` declarations and temporary stub implementations in `fs.c` (currently return `-1` until filesystem sector I/O is implemented). Extended `tests/bios/test_bios.c` to validate disk parameter persistence and READ/WRITE delegation status behavior.
## [2026-03-30] kernel — FSM loop and TM metadata writes
Implemented a first real `kernel_run()` FSM loop with `BOOT`, `IDLE`, `SHELL`, `RUNNING`, `SYSCALL`, and `HALT` handling. `SHELL` and `RUNNING` execute one `cpu_step()` per loop, update `steps`, and route BIOS syscall triggers through `KS_SYSCALL` when CPU `OUT` port `0x01` is pending.
Added BIOS output draining in-loop via `bios_pending_output()` / `bios_get_output()` and host `putchar`.
Kernel now writes TM metadata each tick: state byte to `mem[0xFF00]`, low 32 bits of `steps` to `mem[0xFF01..0xFF04]`, and a placeholder zeroed 32-byte dirty-page map at `mem[0xFF10..0xFF2F]`.
BOOT currently places a temporary `HLT` at `0x0100` and jumps there so the stub system exits deterministically until shell loading is implemented.
## [2026-03-30] kernel — Periodic `/tmp` tape and meta snapshots
Added periodic snapshot writing in `kernel_run()` every `TAPE_SNAP_INTERVAL` ticks (defined as `1000` in `kernel.h`), plus a final flush on exit.
`/tmp/turingos_tape.bin` is written as the raw 64KB tape image from `mem_raw()`. `/tmp/turingos_meta.bin` is written as a compact 64-byte header with state (`[0]`), step count LE (`[1..4]`), dirty map bytes copied from `mem[0xFF10..]` (`[5..36]`), and current PC high/low (`[37..38]`).
This keeps visualizer-facing output stable while dirty-map granularity remains the current 32-byte (256-page) in-memory representation.
## [2026-03-30] fs — Disk geometry validation and sector I/O
Implemented `fs_init()` to open a disk image in read/write mode, verify exact CP/M geometry size (`77 * 26 * 256 = 512512` bytes), and retain an internal file handle for sector operations.
Implemented `fs_read_sector()` / `fs_write_sector()` with track/sector validation (track `0..76`, sector `1..26`), deterministic offset mapping, and 256-byte fixed sector transfers.
Added `tests/fs/test_fs_sector.c` and integrated it into `tests/run_tests.sh` to verify valid init, read/write round-trip, out-of-range rejection, and geometry rejection for malformed disk files.
## [2026-03-30] fs — Directory scan and fs_open handle allocation
Added in-memory directory refresh logic that reads the first 2048 bytes (64 x 32-byte entries) and marks active records where status byte is `0x00`.
Implemented `fs_open()` name matching against CP/M 8.3 directory entries by normalizing input names to uppercase, space-padded 11-byte form, then returning a handle from a 16-slot static open table.
Extended `tests/fs/test_fs_sector.c` to create active directory entries on disk and verify `fs_open()` success for existing files, rejection for missing/invalid names, and handle-range behavior.
## [2026-03-30] fs — Sequential read/write through allocation map
Implemented per-handle file cursor state and wired `fs_read()` to resolve sequential reads through directory allocation-map entries (`entry[16..31]`) in 256-byte sector chunks.
Implemented `fs_write()` with on-demand free-sector allocation, read-modify-write for partial sectors, allocation map updates, and directory entry persistence; file size tracking uses `entry[15]` as sectors used for the current extent.
Extended `tests/fs/test_fs_sector.c` with a `DATA.BIN` round-trip using `fs_write` then `fs_read`, verifying exact payload recovery and EOF behavior once the handle cursor reaches file size.
## [2026-03-30] fs — List/delete/exists API behavior
Implemented `fs_list()` by scanning active directory entries and formatting CP/M 8.3 names into host strings (`NAME.EXT`, uppercase, no trailing spaces). Return value now reports total active entry count.
Implemented `fs_exists()` using normalized CP/M name matching against refreshed directory state.
Implemented `fs_delete()` to mark entry status `0xE5`, clear allocation map bytes for the extent, persist the directory entry, and invalidate any open handles pointing at that directory slot.
Extended `tests/fs/test_fs_sector.c` to verify list ordering/content, existence checks, successful delete semantics, and failure on repeated delete attempts.
## [2026-03-30] fs — Dirty tracking and explicit flush semantics
Introduced filesystem dirty-state tracking so write paths (`fs_write_sector` and directory-entry persistence) mark the backing disk as dirty instead of forcing immediate `fflush` on each update.
Updated `fs_flush()` to flush only when dirty and clear the dirty flag on successful sync, aligning behavior with "write dirty sectors back" semantics while preserving existing correctness through explicit flush points and file close.
## [2026-03-30] tests/fs — Covered create/readback and list/delete flows
Confirmed `tests/fs/test_fs_sector.c` now explicitly covers both remaining Phase 1 filesystem test goals: (1) create/open-write-close-reopen-readback-compare (`DATA.BIN`) and (2) directory list/delete verification (`fs_list`, `fs_delete`, and `fs_exists` checks).
No additional FS test binary was required because the existing FS test suite already executes these assertions under `make test`.
## [2026-03-30] shell — Command parsing foundation for Phase 2
Replaced the shell stub with a concrete shell core API in `src/shell/shell.c`/`shell.h`: prompt rendering (`A> `), command parsing to a typed command enum, argument capture, and minimal render behavior (`help`, unknown command `?`, generic placeholder response for recognized commands).
This intentionally establishes the command-dispatch skeleton for upcoming command-specific tasks (`dir`, `type`, `run`, `cc`, `del`, `cls`, `mem`, `halt`) without prematurely implementing their filesystem/kernel side effects.
## [2026-03-30] shell — Implemented `dir` and `type` through FS APIs
Added concrete handlers in `shell_render_result()` for `dir` and `type <file>` commands. `dir` initializes the filesystem against `disk.img`, lists active entries through `fs_list`, and prints one name per line (or `(empty)` when no files are present).
`type` now validates an argument, opens the requested file via `fs_open`, streams bytes with repeated `fs_read` calls, and writes content to the shell output stream. Both commands use `?` as an error response on missing args or FS/open failures.
## [2026-03-30] shell — Implemented `run <file>` TPA loader hook
Extended shell state with a run-request handshake (`run_requested`, `run_entry`) and updated `run` command handling to load file bytes from FS into TPA starting at `0x0100` via `mem_write`.
After successful load, shell now sets `run_requested=1` and `run_entry=0x0100`, providing a deterministic hook for kernel integration to set `cpu.pc` and transition to `KS_RUNNING`.
## [2026-03-30] shell — Added cc/del/cls/mem/halt command handlers
Implemented `cc` command wiring to `cc_compile(src, out)` with `.com` output path derivation from the provided source name.
Implemented `del` through filesystem APIs (`fs_init("disk.img")` + `fs_delete`), `cls` via ANSI clear sequence, and `mem` as a fixed tape-region summary aligned to the spec map.
Added a halt handshake flag (`halt_requested`) so `halt` command can signal kernel-facing termination intent while still producing shell output.
## [2026-03-30] shell/build — Line editing and shell.com build path
Added shell line-editing support with explicit backspace handling (`0x08` and `0x7F`) via `shell_line_push_char`, bounded 128-character input buffering, newline completion, and reset/access helpers.
Replaced the `make shell` placeholder flow with a host compile pipeline: `tools/cc_driver.c` now calls `cc_compile` to generate `bin/shell.com` from `src/shell/shell.c`.
Updated `cc_compile` stub to emit a deterministic minimal `.com` binary (includes source checksum bytes and `HLT`) so shell build and command wiring are functional until the full compiler backend is implemented.
## [2026-03-30] kernel boot — BIOS vectors and shell.com loading
Updated BOOT state to initialize `mem[0x0000..0x00FF]` (BIOS vector region) with a defined table image and to load `bin/shell.com` bytes into TPA starting at `0x0100` from the host filesystem.
BOOT now sets `cpu.pc = 0x0100` and transitions to `KS_SHELL` directly. If shell loading fails, BOOT falls back to a `HLT` at `0x0100` to preserve deterministic termination in current stub mode.
## [2026-03-30] integration — Added Phase 2 boot/halt scripts
Added `tests/integration/test_boot.sh` and `tests/integration/test_halt.sh` and wired them into `tests/run_tests.sh`.
Given the current stub runtime still exits immediately with `state=5`, integration assertions currently validate successful boot execution and HALT termination via `build/turingos` output checks; shell-interactive I/O assertions will need tightening once kernel-shell command loop is fully wired.
## [2026-03-30] build/shell — Keep generated disk image under build/
Adjusted disk generation so `mkdisk` now writes to `build/disk/disk.img` (and `make disk` ensures the parent folder exists) instead of emitting `disk.img` in the repo root.
Updated shell filesystem command handlers to use the same build-scoped disk path, keeping runtime-generated artifacts out of the top-level repository workspace.
## [2026-03-30] repo hygiene — Artifact locations normalized
Clarified artifact policy in spec/progress: generated disk image lives in `build/disk/disk.img` and generated shell binary lives in `bin/shell.com`, both treated as non-source artifacts.
Expanded `.gitignore` guidance to keep generated outputs out of commits, with `build/` as default sink and `bin/shell.com` explicitly ignored.
## [2026-03-30] repo hygiene — Shell binary moved under build/
Standardized all generated artifacts under `build/` by moving shell binary output from `bin/shell.com` to `build/bin/shell.com`.
Updated producer/consumer paths (`make shell`, kernel boot loader, spec/progress docs) so runtime loading now targets `build/bin/shell.com`; `.gitignore` now relies on `build/` instead of a separate top-level `bin` rule.
## [2026-03-30] compiler/parser — Recursive-descent AST pass integrated into compile flow
Implemented `cc_parse()` in `src/compiler/compiler.c` and exposed it in `compiler.h`. The parser now consumes lexer output and builds an AST node pool covering top-level declarations/functions, variable declarations/assignments, expression precedence (`||`..unary), calls, and control statements (`if/else`, `while`, `for`, `return`).
The parser is now executed from `cc_compile()` after lexing; compile fails early on parse errors instead of silently emitting a `.com` for invalid source. Kept node storage static-array based in the existing host compile pass to stay deterministic and avoid broad compiler-architecture changes while codegen remains pending.
## [2026-03-30] compiler/codegen — Initial AST walker emits real 8080 bytes
Added a first `cc_codegen` path in `src/compiler/compiler.c` that walks parsed AST nodes and emits 8080 machine code into a flat `.com` byte buffer rather than always writing a checksum stub.
Current supported subset is intentionally narrow: function body block traversal, `return` statements, literal expressions, and `+`/`-` binary expressions (result in register `A`), ending with `HLT`. To preserve existing `make shell` and broad-source compatibility during incremental compiler bring-up, `cc_compile` now falls back to the previous deterministic checksum-stub emission when unsupported AST shapes are encountered.
## [2026-03-30] compiler/codegen — Added symbol-backed variable load/store
Extended the code generator to resolve identifier nodes via token/source slices, maintain a fixed-size symbol table, and emit `LDA/STA` for identifier reads and assignments. Variable declarations now allocate symbols and store either initializer values or zero defaults.
Top-level variable declarations are now reserved at fixed addresses starting at `0x2000` (above TPA entry), satisfying the current global-variable placement milestone while keeping local stack-based variables and control-flow lowering for subsequent tasks.
## [2026-03-30] compiler/codegen — Local variables lowered as SP-relative storage
Added distinct global/local symbol tables in codegen and switched local declaration resolution to SP-relative slots. Local identifier loads/stores now emit `LXI H,offset; DAD SP; MOV A,M` and `MOV B,A; LXI H,offset; DAD SP; MOV M,B` patterns, while globals continue to use absolute `LDA/STA`.
Codegen now emits a simple function prologue `LXI SP,0xFDFF` before function-body lowering so local slots are anchored to the 8080 stack region. This advances local-variable support while preserving existing fallback behavior for still-unsupported control-flow and call semantics.
## [2026-03-30] compiler/codegen — Added `*` and `/` arithmetic lowering
Extended binary expression codegen to cover multiplication and division in addition to existing `+`/`-`. Multiplication is lowered as repeated addition in a generated loop; division is lowered as repeated subtraction with quotient accumulation and explicit divide-by-zero result `0`.
Added internal jump placeholder/patch helpers for short forward branches in emitted machine code, enabling loop/control labels during expression lowering while keeping the rest of compile flow unchanged and fallback-safe for other unsupported AST constructs.
## [2026-03-30] compiler/codegen — Comparison ops emit boolean via branch lowering
Added codegen for relational/equality binary operators (`==`, `!=`, `<`, `<=`, `>`, `>=`) by lowering to `CMP` plus conditional jumps, with result canonicalized into `A` as `0` or `1`.
This establishes the comparison + conditional-branch substrate needed by upcoming control-flow statement lowering (`if`/`while`/`for`) while keeping the current compile-time fallback behavior for unrelated unsupported AST nodes.
## [2026-03-30] compiler/codegen — Lowered `while` and `for` statements
Implemented statement-level loop lowering for `CC_AST_WHILE` and `CC_AST_FOR` in `cg_stmt()`. Both now emit loop labels, evaluate condition expressions through a shared `jump-if-zero` helper, branch to loop-exit labels, and jump back to the loop head.
`for` lowering now handles initializer, condition, body, and step in order (using parser field mapping: `left/init`, `right/cond`, `next/body`, `third/step`) and preserves early-return propagation so generated code does not append dead loop-back jumps after a terminal return.
## [2026-03-30] compiler/codegen — Multi-function CALL/RET emission pipeline
Refactored codegen to emit a true function table: collect top-level function declarations, emit a small entry stub (`LXI SP`, `CALL main`, `HLT`), then emit each function body with `RET`-based returns and default epilogues when no explicit return appears.
Added `CC_AST_CALL` expression lowering with address patching for forward calls: unresolved call sites are recorded and patched after all function addresses are known. `CC_AST_RETURN` now emits `RET` instead of `HLT`, enabling nested function calls while preserving program termination via entry-stub `HLT` after `main` returns.
## [2026-03-30] compiler/codegen — BIOS I/O wrappers for putchar/getchar/puts
Added built-in call lowering for `getchar()`, `putchar(x)`, and `puts("...")` directly in `CC_AST_CALL` codegen using BIOS trigger convention (`MVI A,<fn>; OUT 0x01`). `putchar` now evaluates its argument, routes it via register `C` to BIOS CONOUT (`0x02`), and preserves the character in `A` as return value; `getchar` invokes BIOS CONIN (`0x01`) and leaves result in `A`.
Implemented literal-string emission for `puts` by decoding the string token payload (basic escapes handled: `\\n`, `\\r`, `\\t`, `\\\\`, `\\\"`, `\\'`) and issuing repeated CONOUT calls followed by newline output, with `A=0` return. Non-literal or malformed `puts` forms currently fail codegen and fall back to the deterministic compile stub path.
## [2026-03-30] compiler/tests — Flat `.com` guardrails and parser coverage
Hardened `cc_compile()` output constraints by expanding the internal emission buffer and explicitly rejecting invalid code lengths (`<=0` or beyond buffer bounds) before writing output, preserving strict flat-binary `.com` semantics (raw emitted bytes only, no headers).
Added `tests/compiler/test_parser.c` and wired it into `tests/run_tests.sh` to validate parser acceptance/rejection across representative valid and invalid subset-C programs (declarations, assignment, arithmetic precedence, calls, `if`, `while`, and `for` forms). `make test` now includes parser regression coverage via `PASS: test_parser`.
## [2026-03-30] compiler/tests — Runtime codegen execution harness
Added `tests/compiler/test_codegen_runtime.c` and integrated it into `tests/run_tests.sh` as `PASS: test_codegen_runtime`.
The new harness writes small C programs to `build/tests/compiler/cases`, compiles each with `cc_compile`, loads the produced binary into emulator memory, executes CPU steps while dispatching BIOS `OUT 0x01` calls, captures BIOS output, and asserts expected results (return value in register `A` and/or stdout text).
Current runtime coverage validates arithmetic return values, while-loop lowering, function call/return, multiplication/division lowering, and BIOS string output via `puts`, providing end-to-end compiler-to-emulator verification for the implemented subset.
## [2026-03-30] compiler/codegen — Added if/else, unary, and logical lowering
Extended codegen statement handling with `CC_AST_IF` lowering (condition branch to else/end, optional else block, and conservative return-propagation only when both branches return).
Extended expression lowering with `CC_AST_UNOP` support for unary `+`, unary `-`, and logical `!` (boolean normalization + invert), and `CC_TOK_AND_AND` / `CC_TOK_OR_OR` lowering for boolean logical ops.
Expanded runtime compiler tests in `tests/compiler/test_codegen_runtime.c` to cover `if/else`, logical operators, and unary operators in addition to the prior arithmetic/loop/call/BIOS coverage; full `make test` remains green.
## [2026-03-30] viz/tape_bridge — Added tape mmap reader foundation
Created `viz/tape_bridge.py` with a read-only mmap bridge for `/tmp/turingos_tape.bin` at fixed `64KB` size.
Implemented lazy-open + handle reuse (`get_tape_map()`), snapshot reads (`get_tape()`), and explicit teardown (`close()`), with graceful `None` returns when the tape file is absent or undersized.
Path resolution uses `os.path.realpath('/tmp/...')` to avoid macOS `/tmp` vs `/private/tmp` mismatches documented in project pitfalls.
## [2026-03-30] viz/tape_bridge — Added metadata mmap parsing
Extended `viz/tape_bridge.py` with a second read-only mmap for `/tmp/turingos_meta.bin` (`64` bytes) and lazy-open access via `get_meta_map()`.
Added `get_meta()` parsing to extract state (`raw[0]`), step count (`raw[1..4]` little-endian), dirty map bytes (`raw[5..36]`), and program counter (`raw[37..38]` high/low).
The bridge now exposes parsed kernel metadata in a visualizer-friendly dictionary while preserving graceful `None` behavior if the meta file is missing or too small.
## [2026-03-30] viz/tape_bridge — Added explicit PC/state accessors
Added `get_pc()` and `get_state()` helpers on top of `get_meta()` so visualizer code can read high-frequency fields directly without duplicating parsing logic.
Both helpers preserve graceful behavior by returning `None` when metadata is unavailable, matching the bridge contract used by `get_tape()`/`get_meta()`.
## [2026-03-30] viz/tape_bridge — Hardened graceful failure paths
Updated tape/meta mmap open paths to catch filesystem access errors (`exists/getsize/open`) and mmap setup failures, returning `None` instead of raising.
Added read-time guards in `get_tape()` and `get_meta()` so seek/read failures on stale or invalid maps trigger handle reset and graceful `None` returns.
This completes the bridge-side "missing/inaccessible files should not crash visualizer" requirement while keeping the API stable.
## [2026-03-30] viz/visualizer — Pygame window bootstrap
Replaced the visualizer stub with a minimal pygame app shell: initializes pygame, opens a `1280x800` window, sets the title to `TuringOS Tape Visualizer`, clears to black each frame, and exits cleanly on window close.
Added a small import guard that prints a clear message and exits non-zero if `pygame` is missing, keeping local failure behavior explicit while preserving Docker/venv workflows.
## [2026-03-30] viz/visualizer — Added tape map grid renderer
Implemented the first map panel pass in `viz/visualizer.py`: draws a full `256x256` tape grid using `4x3` pixel cells (`1024x768` area) with top-left offset, and updates it each frame from `tape_bridge.get_tape()`.
Added a safe fallback to an all-zero tape when bridge data is unavailable so visualizer remains runnable before kernel snapshots exist.
Current cell coloring is temporary byte-intensity green; region-aware palette/highlights are deferred to subsequent visualizer tasks.
## [2026-03-30] viz/visualizer — Region-based tape color mapping
Replaced temporary byte-intensity coloring with memory-region colors aligned to spec ranges: BIOS, TPA, kernel heap, shell workspace, FS cache, stack, I/O ports, and TM metadata.
The tape map renderer now picks color from cell address range (`_region_color(addr)`) so the memory layout is visually stable and immediately readable independent of byte contents.
## [2026-03-30] viz/visualizer — Added blinking PC highlight
Extended tape map rendering to overlay the current program-counter cell in bright yellow, sourcing PC from `tape_bridge.get_pc()`.
Implemented a simple blink phase using pygame tick time (`500ms` cadence), so the PC marker alternates visible/hidden while preserving the underlying region color when off-phase.
## [2026-03-30] viz/visualizer — Dirty-page flash with decay
Added dirty-map driven flash logic using metadata `dirty_map` bytes (`32` bytes => `256` page bits). Newly dirty pages are set to a short flash timer and then decay toward base region color over time.
Implemented per-page decay tracking and color blending (`DIRTY_FLASH_COLOR` over region color), so all cells within a dirty 256-byte page briefly brighten and smoothly fade.
Current kernel dirty map is still mostly placeholder-zero, but the visualizer path is now wired for real dirty activity without further structural changes.
## [2026-03-30] viz/visualizer — Added page detail panel (hex + ASCII)
Implemented a right-side detail panel that renders the selected 256-byte page as 16 rows of `16` bytes with both hex and ASCII columns.
Current page selection defaults to the current PC page (`pc >> 8`), so the detail view follows execution even before click-selection (`P4-14`) is added.
Panel rendering includes header address range and non-printable-byte substitution to `.` for stable ASCII output.
## [2026-03-30] viz/visualizer — Added bottom legend bar
Implemented a bottom legend panel with color swatches and labels for all current map regions (`BIOS`, `TPA`, `KERNEL`, `SHELL`, `FS CACHE`, `STACK`, `I/O`, `TM META`) plus `PC` marker color.
Added a live head readout (`HEAD=0x....`) on the legend bar to make PC location explicit even when zoomed or blinking overlay is off-phase.
## [2026-03-30] viz/visualizer — Added top status bar
Added a status text line at the top showing current kernel state name and step count from bridge metadata (`state`, `steps`).
Included state-name mapping (`BOOT`, `IDLE`, `SHELL`, `RUNNING`, `SYSCALL`, `HALT`) with an `UNKNOWN(n)` fallback for robustness against unexpected values.
## [2026-03-30] viz/visualizer — Keyboard controls for reset/pause/snapshot
Added key handlers: `P` toggles pause (freezes bridge polling while keeping render alive), `R` closes bridge mappings so the next frame reopens snapshots, and `S` writes a tape snapshot file (`snapshot_XXXX.bin`) in the current working directory.
Snapshot naming is monotonic-first-available (`snapshot_0000.bin`, `snapshot_0001.bin`, ...), and snapshot payload is always the currently displayed 64KB tape buffer.
## [2026-03-30] viz/visualizer — Click-to-select detail page
Added left-click handling on the tape map area to select a 256-byte page for the detail panel using map-coordinate-to-page conversion.
When a page is selected, detail view remains locked to that page; otherwise it follows the PC page. `R` reset now also clears manual selection back to PC-follow mode.
## [2026-03-30] viz/visualizer — Waiting splash when tape unavailable
Added a dedicated splash render path that shows `WAITING FOR TAPE...` when no valid 64KB tape snapshot is available from the bridge.
While waiting, the visualizer polls bridge data on a lightweight interval (`~1000ms`) instead of pulling snapshots every frame, then resumes full panel rendering once tape data appears.
## [2026-03-30] viz/visualizer — Finalized frame pacing and launch path
Set visualizer frame pacing to retro target `10 FPS` via `clock.tick(TARGET_FPS)` with `TARGET_FPS=10`.
Confirmed existing `make viz` target already launches `python3 viz/visualizer.py`, so no Makefile changes were required to satisfy the launch requirement.
## [2026-03-30] tests — Added shared test framework header
Created `tests/testfw.h` providing shared `ASSERT`, `TEST`, and `RUN_ALL_TESTS` macros for the hand-rolled C test harness style.
Migrated `tests/emu/test_mem.c` to consume the framework macros (single test function + `TEST(...)` invocation) so the header is actively exercised in the default `make test` flow.
## [2026-03-30] tests/emu — Aligned emulator test paths with Phase 5 checklist
Added `tests/emu/test_flags.c` (flag edge-case coverage for ADD/SUB/AND/OR/XOR boundaries) and updated `tests/run_tests.sh` to build/run it as `test_flags`.
Existing `tests/emu/test_opcodes.c` and `tests/emu/test_mem.c` remain in the default runner, so the Phase 5 emulator test trio now matches both file paths and execution wiring in the tracker.
## [2026-03-30] tests/bios+fs — Aligned file paths to checklist names
Added `tests/bios/test_conout.c` and `tests/fs/test_fs.c` (path-aligned variants of existing BIOS ring-buffer and filesystem create/read/write/delete coverage) and wired `tests/run_tests.sh` to compile/run these exact file names.
`make test` now reports `PASS: test_conout` and `PASS: test_fs`, matching Phase 5 tracker wording while preserving existing behavior and coverage scope.
## [2026-03-30] tests/compiler — Added non-empty output compile test
Added `tests/compiler/test_cc.c` to compile a representative set of small C programs via `cc_compile` and assert each generated `.com` output file exists with non-zero size.
Wired the new test into `tests/run_tests.sh` as `PASS: test_cc`, giving explicit coverage for the checklist requirement that compile outputs are non-empty.
## [2026-03-30] tests/compiler — Added `.expected`-based emulator output checks
Added five checked-in compiler fixtures under `tests/compiler/programs/` (`add`, `strcat`, `count`, `memtest`, `logic`) with paired `.c` and `.expected` files.
Added `tests/compiler/test_expected_outputs.c` to compile each fixture, run the emitted binary on the emulator with BIOS output capture, and compare stdout against the corresponding `.expected` file.
Integrated the new harness into `tests/run_tests.sh` as `PASS: test_expected_outputs`.
## [2026-03-30] tests/integration — Added checklist integration scripts and case filtering
Extended `tests/compiler/test_expected_outputs.c` with optional `CC_CASE=<name>` filtering and added `echo` fixture/input support (`echo.c`, `echo.expected`, staged stdin `"hello\n"`).
Added integration scripts `tests/integration/test_add.sh`, `test_strcat.sh`, `test_count.sh`, `test_echo.sh`, and `test_memtest.sh`; each invokes the expected-output harness for one named case and reports a dedicated PASS line.
Updated `tests/run_tests.sh` to execute the expanded integration script set after compiler test binaries, so `make test` now reports all Phase 5 integration checklist items in one pass.
## [2026-03-30] Makefile — Removed broken `agents` target
`tools/agent_supervisor.py` is no longer in the repo; removed the `agents` Makefile target and its `.PHONY` entry so there is no recipe that fails at runtime.
## [2026-03-30] Shell + compiler — TPA shell, code relocation, locals fix
Replaced the non-compilable host `shell.c` with `src/shell/shell_tpa.c` (tiny C only: `A> ` prompt, `halt` via `puts("HALT")` + return, `?` for unknown lines), `make shell` wired to it, and dropped `shell.c` from the host `turingos` link (it was unused and pulled host filesystem APIs).
Codegen now emits **absolute** addresses in the TPA (`0x0100 +` file offset) for `CALL`/`JMP`/`JZ` patches and loop back-edges; emulator tests load `.com` at `0x0100` with `PC=0x0100` to match.
Fixed **local stack slots**: first local was at SP offset `0` and overwrote the `CALL` return address; locals now start at offset `2` above SP.
`make test` depends on `make shell`; `test_boot.sh` / `test_halt.sh` use `</dev/null` so they never block on `getchar` when stdin is a TTY.
## [2026-03-30] BIOS 0x0F listdir + shell `dir` + fs at boot
Added BIOS syscall `A=0x0F` (`OUT` port 1) that calls `fs_list` and queues each filename with newlines, or `(empty)\n` when there are no files, or `?\n` when the filesystem is unavailable.
Kernel `kernel_init` now calls `fs_init("build/disk/disk.img")` (ignored on failure). Tiny C gained a `listdir()` intrinsic (same OUT pattern as `getchar`). `shell_tpa.c` recognizes `dir` (d,i,r + newline) and calls `listdir()`. `make test` depends on `make disk` so integration can assert `(empty)` on a fresh image; added `tests/integration/test_dir.sh` piping `dir\n` into `turingos`.
## [2026-03-30] Shell help/mem/cls + larger cc_compile buffer
TPA shell now implements `cls` (ESC [2J ESC [H via putchar), `help` (one-line command list), and `mem` (single `puts` with embedded newlines for the tape map). `halt` vs `help` split on the second letter (`ha…` vs `he…`). `cc_compile` raised limits (`src_buf` 16KiB, 4K tokens, 8K AST nodes) so large sources like `shell_tpa.c` are not truncated. Added `tests/integration/test_help.sh` and `test_mem.sh`.

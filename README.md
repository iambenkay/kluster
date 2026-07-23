# kluster

Disclaimer: This README is AI-generated but was thoroughly reviewed and edited. The actual source code of this project is purely hand-written.

## Introduction

A small bare-metal AArch64 operating system, written in Rust, targeting the
**Raspberry Pi 5** (BCM2712) with a QEMU `raspi4b` build for host-side
development.

kluster boots on real hardware and in QEMU, brings up an HDMI framebuffer via
the VideoCore mailbox, and draws graphics + text through its own on-screen
console — no bootloader beyond the Pi firmware, no host OS, no `std`.

---

## Working today

### Boot & CPU

- Custom AArch64 boot path (`_start`) linked at the Pi firmware's load address
  (`0x80000`) via a hand-written linker script.
- Boot-core selection: parks the three secondary cores in `wfe` and runs the
  kernel on core 0 only.
- Zeroes BSS, sets up the boot stack, and enters Rust.
- Panic handler halts the CPU (`wfe`); no console output on panic yet.

### Graphics

- **VideoCore mailbox driver** — property-tags channel (channel 8) request /
  response with polling.
- **HDMI framebuffer setup** via the mailbox: physical + virtual size, depth,
  pixel order, framebuffer allocation, pitch — all read back from the
  firmware and used at runtime.
- **Auto pixel-format detection.** Runtime `bpp = pitch / width` handles the
  Pi 5's actual **16-bit RGB565** framebuffer and QEMU's 32-bit framebuffer
  from the same code path.
- **Drawing primitives** (all `i32` coordinates with on-screen clipping;
  off-screen vertices are silently dropped):
  - `clear(color)` — linear fill of the whole buffer.
  - `draw_pixel(x, y, color)`
  - `draw_rect(x1, y1, x2, y2, color)` — auto-clipped, order-agnostic.
  - `draw_triangle(x0, y0, x1, y1, x2, y2, color)` — half-space rasterizer,
    both windings.
- **Color** — RGBA-taking `Color` with per-target packing (RGB565 on the Pi,
  BGRA on QEMU) hidden behind `to_rgb()` / `to_rgb565()`.

### Text

- **8×8 bitmap font** covering uppercase `A–Z`, digits `0–9`, and `:`.
- Scalable glyph rendering (`draw_char` / `draw_str`) that paints white glyph
  - black cell so re-drawing self-clears.
- **Framebuffer-backed `println!` console**: cursor state in the module,
  handles `\n` / `\r`, folds lowercase to uppercase, wraps at screen width,
  wraps back to the top when it runs out of vertical space.

### Build & run

- Two build profiles behind Cargo features (see [Feature flags](#feature-flags)).
- `cargo make` tasks:
  - `compile` / `compile-emulate` — objcopy the ELF into a flat
    `kernel_2712.img`.
  - `sync` — copy the image to a mounted SD card at `/Volumes/bootfs`.
  - `emulate` — build for QEMU and launch it.

---

## Feature flags

kluster uses Cargo features to select between hardware targets. They come in two axes:
**where you're running** and **which SoC generation**. Combine them to match your target.

| Feature    | What it selects                                              |
| ---------- | ------------------------------------------------------------ |
| `device`   | Running on real Pi hardware (framebuffer format + defaults). |
| `emulator` | Running under QEMU (implies `ltr-rgb`).                      |
| `rpi5`     | BCM2712 addresses: mailbox at `0x10_7C01_3880`, A76 tuning.  |
| `rpi4`     | BCM2711 addresses: mailbox at `0xFE00_B880`, A72 tuning.     |
| `ltr-rgb`  | Pack colors as ARGB `[r, g, b, a]` (QEMU 32bpp format).      |

The default (`device` + `rpi5`) is the Pi 5 hardware build.
For QEMU you turn defaults off and pick the emulator pair:
`--no-default-features --features emulator,rpi4`.

### Recommended combinations

| Where I'm running | Cargo flags                                      |
| ----------------- | ------------------------------------------------ |
| Raspberry Pi 5    | _(default)_ — same as `--features device,rpi5`   |
| QEMU `raspi4b`    | `--no-default-features --features emulator,rpi4` |

`cargo make sync` and `cargo make emulate` bake the right combination in, so
in day-to-day use you don't pass features by hand.

### Adding a new target

If you want to bring up another board (say a Pi 4 for real):

1. Add a new `device`-style feature if the color/depth quirks differ (or
   reuse `device` if identical).
2. Extend the `cfg` splits in `mailbox.rs`, `hdmi.rs`, `color.rs` as needed.
3. Add a matching `cargo make` task with the right `RUSTFLAGS` and features.

---

## In development

These are on the roadmap and in various states of design — not shipping yet.

### 🗂 FAT32 read-only filesystem

A `BlockDevice` trait with a RAM-backed impl first (embedded disk image via
`include_bytes!`), then the same trait implemented by a real SD driver. Lets
the kernel open and read files by path (`/HELLO.TXT`) and print their
contents through the console. Short 8.3 names first; LFN later.

### ⚠️ Exception vectors

Install a `VBAR_EL1` table so CPU faults (data aborts, unaligned access,
instruction faults) print a diagnostic instead of silently hanging.

### 🧠 MMU + caches

Set up page tables, enable the MMU, and mark RAM as Normal cacheable memory.

### ⏱ Timers & interrupts

Bring up the ARM generic timer and the GIC-400 on the Pi 5. Enable IRQs.
Foundation for any preemptive scheduling later.

### 🧵 Multicore

Wake secondary cores via **PSCI** (`smc #0` with `PSCI_CPU_ON`), the method
the Pi 5 device tree specifies. Per-core stacks and a per-core entry.

### 💾 SD host controller driver

BCM2712 SDHCI initialization sequence (`CMD0` → `ACMD41` → …), CSD/CID
parsing, high-speed switch, and DMA-based block reads. Drops into the same
`BlockDevice` trait FAT32 targets.

### 🔤 Full ASCII font

Extend the 8×8 glyph table to cover lowercase and common punctuation, so
`println!` can render real messages without the current auto-uppercase fold.

### 🧮 Global allocator (`alloc`)

A small bump / linked-list allocator so the kernel can use `Vec`, `Box`,
`String` after early boot. Enables ergonomic data structures for the
filesystem, scheduler, etc.

### 🖥 Host-testable rasterizer

Refactor the drawing primitives to target a `&mut [u8]` + geometry struct so
the rasterizer runs unchanged inside a `minifb` window on macOS/Linux for
tight visual iteration — no SD-card swap loop required.

### 🖼 Ultrawide 3440×1440 output

`config.txt` tuning (`hdmi_cvt`, `max_framebuffer_*`, `framebuffer_depth=32`)
to get the Pi 5's HDMI to actually output the panel's native resolution
instead of clamping to 1080p. Requires experimenting with BCM2712 firmware
quirks.

---

## Getting started

Prereqs (macOS):

```bash
# rustup + nightly + AArch64 bare-metal target
rustup toolchain install nightly
rustup target add aarch64-unknown-none-softfloat --toolchain nightly
rustup component add llvm-tools-preview --toolchain nightly
cargo install cargo-binutils cargo-make
brew install qemu
```

Run in the emulator:

```bash
cargo make emulate
```

Flash and run on a Pi 5 (SD card mounted at `/Volumes/bootfs`):

```bash
cargo make sync
```

---

## Project structure

```
src/
├── main.rs              // crate root, module declarations
├── kernel.rs            // kernel::main — the platform-agnostic entry
├── color.rs             // Color + per-target packing
├── geometry.rs          // Point + basic geometry types
├── panic_cfg.rs         // #[panic_handler] + eh_personality
├── print.rs             // print! / println! macros
├── types.rs             // shared type aliases
├── processors/
│   └── aarch64/         // arch-specific: boot.S, boot entry, cpu helpers
└── motherboards/
    ├── display.rs       // DisplayBuffer trait — the graphics facade
    └── raspberrypi/     // Pi-specific: mailbox, HDMI framebuffer, console,
                        //   kernel.ld linker script
```

`kernel.rs` talks to `motherboards::display`; the board-specific
implementations live under `motherboards/raspberrypi/`. Same shape for the
CPU: `processors::aarch64` for arch code.

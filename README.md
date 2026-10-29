# ESP32-C3 + Rust + Embassy template

A [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) template for ESP32-C3
firmware on `esp-hal` 1.2, `esp-rtos` and Embassy, with a board abstraction layer, Wi-Fi AP
with web server + DHCP, a CC1101/POCSAG radio service and a small UI channel layer.

<img src="assets/All.png" alt="Web View example">

## Generate a project

```bash
cargo install cargo-generate

# from this repository
cargo generate --path . --name my-firmware

# or from git
cargo generate --git <repo-url> --name my-firmware
```

Only the crate name is parameterized (`{{crate_name}}`). The target chip is fixed to
`esp32c3`, so the toolchain, HAL features, linker flags and `espflash` runner all work
out of the box.

## Requirements

- Rust nightly with the `rust-src` component — pinned in `rust-toolchain.toml`
- [`espflash`](https://github.com/esp-rs/espflash) for flashing and monitoring

## Build, flash, monitor

```bash
cargo build --release
cargo run --release   # flashes via the runner in .cargo/config.toml, then monitors
```

Logging uses `defmt`; the default filter is `DEFMT_LOG=info` (see `.cargo/esp-config.toml`)
and can be overridden from the environment.

## Project layout

| Path | Contents |
|---|---|
| `src/app/main.rs` | `#![no_main]` binary: HAL init, heap, task spawning, UI event loop |
| `src/app/config.rs` | Wi-Fi AP / HTTP / buffer-size constants |
| `src/app/state.rs` | `AppState`, `Settings` (serde) |
| `src/app/shared.rs` | shared `embassy_sync` channels |
| `src/board/` | BSP: pin map per board, selected by cargo features |
| `src/services/` | embassy tasks: `led`, `button`, `radio`, `notifications`, `inet`, `ui`, `pocsag` |
| `src/drivers/` | lower-level driver glue |

## Board variants

Board wiring is selected at compile time:

```bash
# default: ESP32-C3 Super Mini pin map
cargo build --release

# other wiring: edit src/board/board_custom.rs, then
cargo build --release --no-default-features --features board-custom
```

Changing pins, LED polarity or the CC1101 wiring belongs in the board module only —
services receive already-configured pins.

## License

MIT — see [LICENSE](LICENSE).

# fetch-rs

Terminal system information fetch utility written in Rust. Renders an ASCII/Unicode distribution logo as an interactive, rotating 3D relief alongside real-time system metrics.

![Demo](demo.gif)

## Features

- Real-time 3D projection with Blinn-Phong lighting and specular shading.
- Interactive mouse rotation: click and drag the logo in the terminal, release to fling it spinning.
- Native Linux telemetry gathered directly from kernel interfaces (`/proc`, `/sys`, `uname`, pacman/dpkg database) without spawning external shell processes.
- Automatic distribution detection with fastfetch fallback and integrated Gentoo default.
- Zero tearing: double-buffered batched ANSI terminal output with automatic window resize handling (`SIGWINCH`).
- Seamless startup execution: terminates on any keypress and passes through the keystroke to the shell.

## Requirements

- Linux operating system
- `rust` and `cargo` (edition 2024 / rustc 1.85+)
- Optional: `fastfetch` for loading arbitrary distro logos

## Installation

### From Source

```bash
git clone https://github.com/Beforov1337/fetch-rs.git
cd fetch-rs
cargo build --release
```

The compiled binary will be located at `target/release/fetch`.

To install it system-wide:

```bash
sudo install -m 755 target/release/fetch /usr/local/bin/fetch
```

Or for current user only:

```bash
mkdir -p ~/.local/bin
install -m 755 target/release/fetch ~/.local/bin/fetch
```

Make sure `~/.local/bin` is in your `$PATH`.

## Usage

Run the utility:

```bash
fetch
```

To stop execution, press any key or send `Ctrl+C`. The pressed key is preserved in the terminal input queue, allowing `fetch` to be placed in `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` as a startup banner.

### Interactive Controls

- **Left click and drag**: rotate the 3D logo manually in any direction.
- **Release mouse**: logo continues spinning with angular inertia.
- **Any keyboard key**: exit.

### CLI Options

```
Usage: fetch [OPTIONS]

Options:
  -l, --logo <NAME>            Specify logo by distribution name (e.g. arch, debian, nixos, ubuntu, fedora)
      --rotate-x               Lock rotation axis to X only
      --rotate-y               Lock rotation axis to Y only
  -s, --speed <SPEED>          Rotation speed multiplier [default: 1.0]
      --size <SIZE>            Scale logo dimensions [default: 1.0]
      --depth <DEPTH>          Scale 3D relief extrusion depth [default: 1.0]
      --height <HEIGHT>        Override viewport height in terminal rows
      --no-info                Render logo only, omit system metrics
      --no-color               Render in monochrome
      --frames <FRAMES>        Stop automatically after N frames [default: 2000]
      --infinite               Run continuously without frame limit
      --shading-chars <CHARS>  Custom ASCII/Unicode gradient ramp [default: ".,-~:;=!*#$@"]
  -h, --help                   Print help information
  -V, --version                Print version information
```

### Examples

Run with an Arch Linux logo:
```bash
fetch --logo arch
```

Lock rotation to vertical axis and run indefinitely:
```bash
fetch --rotate-y --infinite
```

Display only the rotating 3D logo without telemetry:
```bash
fetch --no-info --size 1.5
```

## Architecture

The project is structured into distinct modules:

- `src/cli.rs`: Command-line interface definitions and flag parsing using `clap`.
- `src/info.rs`: Direct extraction of OS, kernel, uptime, packages, memory, shell, and window manager info.
- `src/logo.rs`: Distribution detection, fastfetch integration, ASCII codepoint density calculation.
- `src/render.rs`: 3D point cloud generation, surface normal vector calculations, Blinn-Phong illumination model, and Z-buffering.
- `src/main.rs`: Terminal raw mode management, event polling, physics loop, and signal handling.

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

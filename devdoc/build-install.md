# Build and install

| Step | What |
|---|---|
| Toolchain | rustup stable (`~/.cargo`). Container: Alpine/musl, host: Mint/glibc -> **build on host** with `install.sh` |
| `./install.sh` | `git pull --ff-only` (if upstream), `cargo build --release`, install to `~/.local/bin/volumio-remote`, write autostart + app `.desktop`, kill old instance, start new |
| Flags | `--no-pull`, `--no-start` |
| Autostart | `~/.config/autostart/volumio-remote.desktop` |
| Host deps (Debian/Mint) | `sudo apt install build-essential pkg-config libfontconfig1-dev libxkbcommon-dev libxcb1-dev` (Slint; list to be verified) |
| Container deps (apk) | `build-base fontconfig-dev freetype-dev libxkbcommon-dev libxcb-dev pkgconf` |

Install is atomic (`mv` over running binary). Instance match: `pgrep -f ^<path>$` (works with procps and busybox).
Tested in container with temp `$HOME`: build, install, restart, single instance.

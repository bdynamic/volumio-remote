# Build and install

| Step | What |
|---|---|
| Toolchain | rustup stable (`~/.cargo`). Container: Alpine/musl, host: Mint/glibc -> **build on host** with `install.sh` |
| `./install.sh` | `git pull --ff-only` (if upstream; failure is a warning), remove legacy `volumiox` files, `cargo build --release`, install to `~/.local/bin/volumio-remote`, write autostart + app `.desktop`, kill old instance, start new |
| Log | `~/.cache/volumio-remote.log` (stderr of app started by install.sh and by autostart) |
| Flags | `--no-pull`, `--no-start` |
| Autostart | `~/.config/autostart/volumio-remote.desktop` |
| Host deps (Debian/Mint) | `sudo apt install build-essential pkg-config libfontconfig1-dev libxkbcommon-dev libxcb1-dev` (verified on the target Mint system) |
| Container deps (apk) | `build-base fontconfig-dev freetype-dev libxkbcommon-dev libxcb-dev pkgconf` |

Install is atomic (`mv` over running binary). Instance match: `pgrep -f ^<path>$` (works with procps and busybox).
Tested in container with temp `$HOME`: build, install, restart, single instance.

## Container notes

- `.cargo/config.toml` links the musl target dynamically (`-crt-static`); static fontconfig fails.
- Extra apk packages for headless tests: `xvfb dbus playerctl xwd imagemagick libx11 libxcursor libxi libxrandr libxkbcommon libxcb font-dejavu`.
- Test run: `Xvfb :99`, `dbus-run-session`, `XDG_CONFIG_HOME` pointing to a config with a test host, `playerctl` for MPRIS, `xwd` + `magick` for screenshots.

- `git pull` failure (e.g. no network) is a warning; build continues with local sources.
- Exited instances show as `<defunct>` in `pgrep` when the parent does not reap them; they are not running. Check `~/.cache/volumio-remote.log` for the exit reason.

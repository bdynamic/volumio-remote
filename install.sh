#!/usr/bin/env bash
# Pull latest, build release, install to ~/.local/bin, enable autostart, (re)start.
# Usage: ./install.sh [--no-pull] [--no-start]
set -euo pipefail

NAME=volumiox
BIN_DIR="${HOME}/.local/bin"
AUTOSTART_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
cd "$(dirname "$(readlink -f "$0")")"

pull=1; start=1
for a in "$@"; do
  case "$a" in
    --no-pull) pull=0 ;;
    --no-start) start=0 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

# 1. pull
if [ "$pull" = 1 ] && git rev-parse --abbrev-ref '@{u}' >/dev/null 2>&1; then
  echo "==> git pull"
  git pull --ff-only
else
  echo "==> skip git pull (disabled or no upstream)"
fi

# 2. toolchain
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
if ! command -v cargo >/dev/null; then
  echo "cargo not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
  exit 1
fi
if command -v apt-get >/dev/null && ! pkg-config --exists fontconfig 2>/dev/null; then
  echo "warn: build deps may be missing. Debian/Ubuntu/Mint:" >&2
  echo "  sudo apt install build-essential pkg-config libfontconfig1-dev libxkbcommon-dev libxcb1-dev" >&2
fi

# 3. build
echo "==> cargo build --release"
cargo build --release

# 4. install
echo "==> install to $BIN_DIR/$NAME"
mkdir -p "$BIN_DIR" "$AUTOSTART_DIR" "$APPS_DIR"
install -m 755 "target/release/$NAME" "$BIN_DIR/$NAME.new"
mv -f "$BIN_DIR/$NAME.new" "$BIN_DIR/$NAME"   # atomic, works while running

cat > "$AUTOSTART_DIR/$NAME.desktop" <<DESK
[Desktop Entry]
Type=Application
Name=VolumioX
Comment=Remote control for Volumio
Exec=$BIN_DIR/$NAME
Icon=multimedia-player
Terminal=false
Categories=AudioVideo;Audio;
X-GNOME-Autostart-enabled=true
DESK
cp "$AUTOSTART_DIR/$NAME.desktop" "$APPS_DIR/$NAME.desktop"
echo "==> autostart: $AUTOSTART_DIR/$NAME.desktop"

# 5. (re)start
if [ "$start" = 1 ]; then
  pkill -f "^$BIN_DIR/$NAME\$" 2>/dev/null && sleep 1 || true
  echo "==> start $NAME"
  nohup setsid "$BIN_DIR/$NAME" >/dev/null 2>&1 &
  sleep 1
  pgrep -f "^$BIN_DIR/$NAME\$" >/dev/null && echo "running" || { echo "failed to start" >&2; exit 1; }
fi

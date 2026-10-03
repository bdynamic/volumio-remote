# Requirements

Origin: reimplementation of VolumioX (https://github.com/majko96/VolumioApp, Qt/C++) with a modern UI, OS media integration and volume knob support. Findings about the original: [volumiox-research.md](volumiox-research.md).

Priority: M = must, S = should, C = could. Status values: **done** = implemented and verified by the user on Cinnamon/X11 (2026-10-03); **done (container)** = verified only in the Alpine test container; **open** = not implemented.

## Functional

| ID | Pri | Requirement | Status |
|---|---|---|---|
| F-01 | M | Show current track title, artist, album | done |
| F-02 | M | Show current volume (%) | done |
| F-03 | M | Show play state (play/pause icon) | done |
| F-04 | M | Play/pause toggle button | done |
| F-05 | M | Previous track button | done |
| F-06 | M | Next track button | done |
| F-07 | M | Volume slider 0-100 | done |
| F-08 | M | Online/offline indicator in the window | done |
| F-09 | M | Poll Volumio state every 2 s and update UI | done |
| F-10 | M | Settings: Volumio host/IP (no `http://` needed) | done |
| F-11 | M | Persist settings in `~/.config/volumio-remote/config` (`host`, `offline_tray`, `theme`, `opacity`) | done |
| F-12 | M | Tray icon (StatusNotifierItem) with Play/Pause, Previous, Next, Show window, Use as default output, Quit | done |
| F-13 | M | Closing the window hides it; tray click shows it | done |
| F-14 | C | Persist window geometry | open |
| F-15 | S | Mute / unmute button | done |
| F-16 | C | Volume +/- via arrow keys in the window | open |
| F-17 | C | Seek/progress bar | open |
| F-18 | S | Cover art in the window, from `getState.albumart` (path on the Volumio host, or absolute URL); loaded in a background thread | done (container) |
| F-19 | C | Visible error message for failed requests | open |
| F-20 | C | Validate host input | open |
| F-21 | M | Modern UI (Slint), same functions as the original | done |
| F-22 | M | Dark and bright design | done |
| F-23 | M | Register as MPRIS2 media player (`org.mpris.MediaPlayer2.volumio_remote`) | done |
| F-24 | M | MPRIS `Play`, `Pause`, `PlayPause`, `Stop`, `Next`, `Previous` control Volumio | done |
| F-25 | M | MPRIS `PlaybackStatus` and `Metadata` (title, artist, album) follow Volumio | done |
| F-26 | M | Keyboard media keys control Volumio through MPRIS | done |
| F-27 | S | MPRIS `Volume` get/set maps to Volumio volume (`playerctl volume`) | done (container) |
| F-28 | S | Keyboard volume knob controls Volumio volume (virtual sink `volumio_remote`, option A) | done |
| F-29 | C | Cover art in MPRIS `mpris:artUrl` | open |
| F-30 | M | Detect availability: a failed `getState` (timeout 2 s) means offline | done |
| F-31 | M | Offline: tray icon grayed out or hidden, window controls disabled, "Volumio offline" shown | done |
| F-32 | M | Offline: release the MPRIS name so media keys go to other players; register again when online | done (container) |
| F-33 | M | Online again: restore everything without restart | done |
| F-34 | S | Offline tray behavior `gray` (default) or `hide` | done |
| F-35 | M | Settings panel (gear) replaces the cover card, so it is never clipped | done |
| F-36 | M | Theme switch Auto / Dark / Light in settings, applied live; Auto follows the GTK theme name, default dark | done |
| F-37 | M | Offline tray behavior switch in settings, applied live | done |
| F-38 | M | Sink volume is synced to Volumio at start and on every change | done |
| F-39 | S | Window opacity slider 30-100 % in settings (alpha on window/card background; needs a compositor) | done |
| F-40 | S | `--diagnose-knob`: prints default sink, sink list and volume; for 15 s forwards each sink event to Volumio and prints the result | done |
| F-41 | M | While the knob is turned (sink events within 1.5 s) Volumio-to-sink sync pauses | done (container) |
| F-42 | M | Stderr is logged to `~/.cache/volumio-remote.log` for autostart and `install.sh` starts | done |
| F-43 | M | Language independent: `pactl` runs with `LC_ALL=C`; sink state from `pactl --format=json`; event filter matches `sink #` | done |
| F-44 | M | CLI fallback for desktop shortcuts: `--toggle --next --prev --vol-up --vol-down --mute` | done (container) |
| F-45 | M | Single instance (lock in `$XDG_RUNTIME_DIR`) | done |
| F-46 | M | `install.sh`: pull, build, install, autostart, restart, remove legacy `volumiox` files | done |
| F-47 | M | Version 1.2.0 (from `Cargo.toml`) shown in settings dialog | done |
| F-48 | M | GPL-3.0-or-later license (`LICENSE`) | done |
| F-49 | S | Cover art corners rounded (about 10.5 px, two thirds of the card radius): image is center-cropped to a square, scaled to 360 px and corners (21 px at 360 px) get an alpha mask in Rust (the Slint software renderer ignores rounded `clip`) | done (confirmed on host) |
| F-50 | S | Title, artist, album scroll horizontally (pause, scroll, pause) when wider than the window; centered otherwise | done (confirmed on host) |
| F-51 | S | Settings footer: copyright "Birk Bremer" and clickable GitHub link (`xdg-open`) | done (confirmed on host) |
| F-52 | S | GitHub Actions workflow `release.yml`, manual start (`workflow_dispatch`) or push of a tag `v*` (tag must equal the Cargo version): test, release build on Ubuntu, tag `v<Cargo version>`, release with `tar.gz` (binary, LICENSE, README) | open (not run yet) |

## Non-functional

| ID | Pri | Requirement | Status |
|---|---|---|---|
| N-01 | M | Runs on Linux desktop; target Cinnamon on X11 | done |
| N-02 | M | Only the Volumio HTTP REST API; nothing to install on the Volumio side | done |
| N-03 | M | Small POC code base, few dependencies, release binary built with size optimization | done |
| N-04 | S | UI never blocks on the network (workers, 2 s timeout) | done |
| N-05 | M | No root needed | done |
| N-06 | C | GNOME/KDE, Wayland | open (untested) |
| N-07 | C | Windows/macOS | open |

## Interfaces

Volumio REST (`http://<host>`):

| Endpoint | Use |
|---|---|
| `/api/v1/getState` | status, title, artist, album, albumart, volume, mute |
| `/api/v1/commands/?cmd=toggle\|play\|pause\|stop\|prev\|next` | playback |
| `/api/v1/commands/?cmd=volume&volume=<0-100>\|mute\|unmute` | volume |
| `<albumart path>` | cover image |

Local: D-Bus session bus (MPRIS, StatusNotifierItem), `pactl` (virtual sink), `gsettings` (theme detection, optional).

## Decisions

| Topic | Decision |
|---|---|
| Name | `volumio-remote` (the original `volumiox` name belongs to another project) |
| Toolkit | Rust + Slint (software renderer, winit/X11); `ksni` tray, `zbus` MPRIS, `ureq` HTTP |
| Volume knob | Option A: virtual sink watched via `pactl subscribe`; fallback C: CLI flags bound to desktop shortcuts. Rejected: B (X11 key grab, collides with the desktop), D (no knob) |
| Availability check | `getState` instead of `ping` (one request, same result) |

## Not yet verified on the user system

Translucent window blending (F-39), `hide` tray mode (F-34), MPRIS name release while offline (F-32).

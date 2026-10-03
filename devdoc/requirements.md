# Requirements (derived from VolumioApp / VolumioX)

Source: https://github.com/majko96/VolumioApp (`mainwindow.cpp`, `settingsui.cpp`).
Priority: M = must (parity with original), S = should, C = could (not in original).
Status: see Implementation status below.

## Functional

| ID | Pri | Requirement | Original behavior |
|---|---|---|---|
| F-01 | M | Show current track title and artist | from `getState`; scrolling text widget for long titles |
| F-02 | M | Show current volume (%) | from `getState` |
| F-03 | M | Show play state (play/pause icon) | icon toggles with state |
| F-04 | M | Play/pause toggle button | `cmd=toggle` |
| F-05 | M | Previous track button | `cmd=prev` |
| F-06 | M | Next track button | `cmd=next` |
| F-07 | M | Volume slider 0-100 | `cmd=volume&volume=<n>` |
| F-08 | M | Show device online/offline indicator | `ping` response -> ON/OFF |
| F-09 | M | Poll state periodically, update UI | 500 ms |
| F-10 | M | Settings dialog with one field: Volumio host/IP | no `http://` prefix |
| F-11 | M | Persist host between runs | `~/.volumiox/data.txt`, plain text |
| F-12 | M | System tray icon with Show / Hide / Exit menu | yes |
| F-13 | M | Minimize to tray; tray click restores window | yes |
| F-14 | M | Persist window geometry/state | `QSettings` |
| F-15 | S | Mute / unmute | in forum description, not seen in code |
| F-16 | S | Volume +/- via arrow keys (step 1) | in forum description |
| F-17 | C | Seek/progress bar | absent in original |
| F-18 | C | Album art | absent in original |
| F-19 | C | Visible error message on failed request | absent in original |
| F-20 | C | Validate host input | absent in original |
| F-21 | M | Modern UI, same functions as F-01..F-16 | new (original: dated Qt Widgets look) |
| F-22 | M | UI follows system light/dark theme | new |
| F-23 | M | Register as media player on OS (Linux: MPRIS2 over D-Bus, name `org.mpris.MediaPlayer2.volumio_remote`) | new |
| F-24 | M | MPRIS methods `Play`, `Pause`, `PlayPause`, `Stop`, `Next`, `Previous` -> Volumio commands | new |
| F-25 | M | MPRIS properties `PlaybackStatus`, `Metadata` (title, artist, album) kept in sync with Volumio state | new |
| F-26 | M | Keyboard media keys (play/pause/next/prev) control Volumio via F-23/24 | new; desktop routes keys to MPRIS player |
| F-27 | S | MPRIS `Volume` property get/set <-> Volumio volume (works with `playerctl volume`) | new |
| F-28 | S | Keyboard volume knob/keys control Volumio volume (see Feasibility) | new |
| F-30 | M | Check Volumio availability periodically (`/api/v1/ping`, interval 2-5 s, timeout <= 2 s) | extends F-08 |
| F-31 | M | Offline: tray icon grayed out (user-switchable to hidden); UI controls disabled, show "offline" | new |
| F-32 | M | Offline: unregister MPRIS service so media keys go to other players; re-register when online | new |
| F-33 | M | Online again: restore icon, controls, MPRIS, resume polling without restart | new |
| F-34 | S | Setting: offline tray behavior `gray` (default) / `hide` | new |
| F-29 | C | Album art in MPRIS `mpris:artUrl` | new |

## Implementation status

Tested in container (Xvfb + private D-Bus) against a real Volumio; screenshots checked.

| Req | Status | Note |
|---|---|---|
| F-01..07, F-09, F-10 | done, tested | Slint window, 2 s poll |
| F-08, F-30 | done | availability = `getState` success (timeout 2 s), not `ping` |
| F-11 | done | `~/.config/volumio-remote/config` (`host`, `offline_tray`, `theme`) |
| F-12, F-13 | done, **untested** | `ksni` tray; container has no StatusNotifierWatcher. Window close hides it. |
| F-14 | open | window geometry not persisted |
| F-15 | done | mute button; Volumio `mute`/`unmute` unverified on device |
| F-16 | open | arrow keys not bound |
| F-21, F-22 | done | Slint UI; dark/light via `gsettings gtk-theme` or `theme=` |
| F-23..F-27 | done, tested | MPRIS via `zbus`; `playerctl` shows metadata/status/volume |
| F-28 | done, **untested** | `sink.rs`; needs `pactl` (absent in container). Tray item "Use as default output" |
| F-31 | done (window, MPRIS tested); tray icon untested | |
| F-32 | done, tested | MPRIS name released offline, re-registered online |
| F-33 | done | |
| F-34 | done, untested | `offline_tray=hide` -> SNI status Passive |
| Fallback C | done | `--toggle --next --prev --vol-up --vol-down --mute` |
| F-17..F-20, F-29 | open | |

## Non-functional

| ID | Pri | Requirement |
|---|---|---|
| N-01 | M | Runs on Linux desktop |
| N-02 | M | Small window, dark theme, frameless optional |
| N-03 | M | No Volumio-side install; uses only HTTP REST API |
| N-04 | M | Single-file/small implementation (POC, see CLAUDE.md) |
| N-05 | S | Command latency < 1 s on LAN |
| N-06 | S | UI never blocks on network (async or short timeout) |
| N-07 | C | Cross-platform (Windows/macOS) |
| N-08 | M | OS media integration must not need root |
| N-09 | S | Works on GNOME, KDE; X11 and Wayland |
| N-10 | M | Target desktop: Cinnamon on X11 (user system); tray via StatusNotifier/XApp |

## Interface (Volumio REST, base `http://<host>`)

| Endpoint | Use | Req |
|---|---|---|
| `/api/v1/ping` | online check | F-08 |
| `/api/v1/getState` | title, artist, volume, status | F-01..03, F-09 |
| `/api/v1/commands/?cmd=toggle` | play/pause | F-04 |
| `/api/v1/commands/?cmd=prev` | previous | F-05 |
| `/api/v1/commands/?cmd=next` | next | F-06 |
| `/api/v1/commands/?cmd=volume&volume=<n>` | set volume | F-07 |
| `/api/v1/commands/?cmd=volume&volume=mute\|unmute` | mute (unverified) | F-15 |

## Feasibility: media keys and volume knob

| Input | Routed by desktop to | Reaches app? | Approach |
|---|---|---|---|
| Play/Pause/Next/Prev keys | active MPRIS player (D-Bus) | Yes, if F-23 done | Register MPRIS2 service |
| Volume knob/keys | default audio sink (PipeWire/PulseAudio), not MPRIS | No, by default | See options below |

Options for F-28 (decide later):

| Opt | Idea | Pro | Con |
|---|---|---|---|
| A | Create virtual null sink "Volumio"; user selects it as default; app watches sink volume (`pactl subscribe` / `pw` API) and forwards to Volumio | Works on X11+Wayland, no root | User must pick sink as default; local audio then silent (fine: Volumio plays remotely) |
| B | Global key grab of `XF86AudioRaiseVolume/LowerVolume/Mute` | Simple | X11 only; Wayland blocks; conflicts with desktop handler |
| C | Desktop custom shortcut calling CLI `volumio-remote --vol-up` | Works everywhere | Manual user setup; overrides default volume keys |
| D | Skip; use `playerctl volume` / app slider only | Zero effort | No knob support |

User system: Cinnamon, X11. MPRIS works (Cinnamon sound applet shows it). Volume keys go to default sink -> A works; B technically possible on X11 but collides with Cinnamon's key binding (rebind in Keyboard settings first).

**Decision: A (virtual sink), fallback C (CLI + Cinnamon shortcut).** Verify on target desktop first (spike).

## Out of scope (original)

Library browsing, queue/playlist editing, source selection, multi-device, auth.

## Notes

- Original runs two timers: 500 ms (state/ping) and 50 ms (UI helpers). 50 ms timer not needed in a rewrite.
- Added requirements F-21..F-29, N-08/09 on user request (modern UI, OS media device, volume knob).
- Original has no license -> reimplement, do not copy code.

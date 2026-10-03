# Requirements (derived from VolumioApp / VolumioX)

Source: https://github.com/majko96/VolumioApp (`mainwindow.cpp`, `settingsui.cpp`).
Priority: M = must (parity with original), S = should, C = could (not in original).
Status: all open.

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

## Out of scope (original)

Library browsing, queue/playlist editing, source selection, multi-device, auth.

## Notes

- Original runs two timers: 500 ms (state/ping) and 50 ms (UI helpers). 50 ms timer not needed in a rewrite.
- Original has no license -> reimplement, do not copy code.

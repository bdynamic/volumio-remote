# Architecture

Single binary, plain threads, one shared state object.

```
 Volumio  <--HTTP-->  volumio.rs  <--  core.rs (state, listeners, cmd_seq)
                                          |  on_change(listener)
              +-----------+---------+-----+------+-------------+
              |           |         |            |             |
           ui.rs       tray.rs   mpris.rs      sink.rs       main.rs
          (Slint)     (ksni SNI) (zbus, own    (pactl, own   (CLI, lock,
                                  tokio thread) subscribe     wiring)
                                                thread)
```

| File | Role |
|---|---|
| `volumio.rs` | blocking REST client (`ureq`, 2 s timeout), `getState` parser, `Cmd` enum, cover download + decode |
| `config.rs` | `key=value` config load/save |
| `core.rs` | `Core`: config, last state (`None` = offline), listeners, polling thread (2 s), `run_cmd` (command thread + refresh) |
| `ui.rs` | Slint window (inline `slint!`), theme/opacity, settings panel, cover loading |
| `tray.rs` | StatusNotifierItem via `ksni`; generated ARGB icon (colored online, gray offline) |
| `mpris.rs` | MPRIS2 root + player interfaces on the session bus; connection exists only while online |
| `sink.rs` | virtual null sink `volumio_remote`, `pactl subscribe` watcher, sink volume <-> Volumio volume |
| `main.rs` | CLI flags, single-instance lock, thread wiring |

## Data flow

| Flow | Path |
|---|---|
| State | poll thread -> `Core::refresh` -> `get_state` -> if changed: all listeners (UI, tray, MPRIS notify, sink sync, cover fetch) |
| Command | UI / tray / MPRIS / knob -> `Core::run_cmd` -> thread: HTTP send -> `refresh` |
| Knob | keyboard -> desktop -> default sink `volumio_remote` volume -> `pactl subscribe` event -> read sink (JSON) -> `Cmd::Volume` |
| Back-sync | Volumio volume change -> listener -> `pactl set-sink-volume` (paused 1.5 s after a knob event) |

## Rules that avoid bugs found in testing

| Rule | Why |
|---|---|
| `cmd_seq` counter: poll results fetched before a command are dropped | stale volume overwrote the new one |
| Knob event pauses back-sync for 1.5 s | intermediate Volumio value moved the sink back |
| `pactl` always with `LC_ALL=C`, JSON output preferred | localized output made the handler skip all events |
| Sink synced to Volumio volume at start | a new sink starts at 100 %, knob-up did nothing |
| Settings is a panel replacing the cover card | inline row was clipped below the window |
| Slint colors via `rgb()` | `#3ecf6e` / `#2e7dff` lex as numbers with exponent |

## Availability

`getState` failure -> `None` state -> window controls disabled, tray gray/hidden, MPRIS connection shut down (`graceful_shutdown`); next successful poll restores all.

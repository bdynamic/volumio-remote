# Reference: voxtype-tray (in `tmp/`, not committed)

Cinnamon SNI tray in Rust, 502 lines, single file. Same desktop as target.

| Item | Finding |
|---|---|
| Tray crate | `ksni 0.3` (`blocking` + `tokio` features, no default features) |
| Other deps | `serde_json`, `toml_edit` (85 crates total in lock, incl. `zbus 5`, `tokio`) |
| Release profile | `opt-level="z"`, `lto`, `strip`, `panic="abort"`, `codegen-units=1` -> **1.7 MB** binary |
| Works on Cinnamon | yes (author runs it there) |

## Patterns to reuse

| Pattern | How | Use in volumio-remote |
|---|---|---|
| Tray as struct impl `ksni::Tray` | `id`, `title`, `icon_name`, `menu` | tray with themed icon names |
| Icon by theme name | e.g. `audio-input-microphone`, `media-record` | online: `multimedia-player`/`media-playback-*`; offline: gray variant (F-31) |
| Live update from threads | `spawn()` returns `Handle`; `handle.update(\|t\| ...)` | worker thread pushes Volumio state into tray (F-09, F-30) |
| Menu | `StandardItem`, `CheckmarkItem`, `RadioGroup`, `SubMenu`, `enabled` flag | Play/Pause/Next/Prev/Show/Quit; disabled when offline |
| Tray host not up at login | retry `spawn()` every 3 s | needed for autostart |
| Single instance | `flock` on `$XDG_RUNTIME_DIR/<name>.lock` (`File::try_lock`) | same |
| Busy flag | disable menu while async action runs | optional |
| Background watchers | plain `std::thread` + sleep loop | ping/getState poll thread |
| Respawn child | loop around `Command` with 2 s sleep | `pactl subscribe` watcher (F-28) |
| Unit tests | pure parse fns tested in `mod tests` | parse `getState` JSON, `pactl` output |

## Gaps (not covered by reference)

| Gap | Plan |
|---|---|
| Grayed icon | no `ksni` greying: switch `icon_name` to a `-symbolic`/disabled themed icon or ship own PNG via `icon_pixmap`; or set `status` to `Passive` (hides) for F-34 `hide` |
| MPRIS | not in reference; `zbus` already in dep tree via `ksni` -> reuse for MPRIS server |
| GUI window | not in reference; Slint |
| Left-click on tray | implement `Tray::activate` to show/hide window |

## Notes

- Container has no `cargo`/`rustc` yet (reference binary was built elsewhere). Install rustup before build.
- Reference uses `std::thread` + blocking handle, not async runtime in app code. Keep same: simple.

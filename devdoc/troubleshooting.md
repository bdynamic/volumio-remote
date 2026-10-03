# Troubleshooting

## Media keys do not control Volumio (volume knob works)

| Item | Detail |
|---|---|
| Symptom | Play/pause/next/prev keys do nothing or control something else; the volume knob (virtual sink) still works |
| Cause | Desktop (Cinnamon) sends media keys to the most recently active MPRIS player. Browsers (seen: Vivaldi) register their own MPRIS player and take the keys. Opening the Volumio web UI from the app (click on the now-playing card) can trigger this |
| Not the cause | App code: MPRIS is independent of the UI; v1.2.0 and v1.3.0 have identical MPRIS code |

Diagnose:

```
playerctl -l                                  # list MPRIS players; volumio-remote must be listed
playerctl -p <volumio-remote name> play-pause # works -> app fine, keys routed elsewhere
pgrep -a volumio-remote                       # more than one instance holds/blocks the MPRIS name
```

Fix (any of):

| Fix | How |
|---|---|
| Stop the browser player | Pause/close the media tab, or quit the browser, then press a media key again |
| Disable browser media keys | Vivaldi: `vivaldi://flags` -> "Hardware Media Key Handling" -> Disabled (Chromium-based browsers have the same flag) |
| Kill duplicate instance | `pkill volumio-remote`, start once |
| Volumio offline at start | No MPRIS player until the first successful poll (F-32); check the window shows "Connected" |

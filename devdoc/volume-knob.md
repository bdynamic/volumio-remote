# Volume knob

The desktop sends volume keys to the default audio sink, not to MPRIS players. Media keys (play/pause/next/prev) do go to MPRIS.

| Option | Idea | Result |
|---|---|---|
| **A (used)** | Virtual null sink `volumio_remote`; user sets it as default output; app mirrors its volume/mute to Volumio | works on X11 and Wayland, no root |
| B | Grab `XF86Audio*` keys | X11 only, collides with the desktop handler |
| C (fallback) | Desktop shortcuts calling `volumio-remote --vol-up/--vol-down/--mute` | manual setup |
| D | No knob | - |

Setup: tray menu "Use as default output". Local audio is silent on that sink, which is fine because Volumio plays remotely.

## Debugging

| Tool | Use |
|---|---|
| `volumio-remote --diagnose-knob` | shows default sink, sink volume, and for 15 s each knob event with what is sent to Volumio and the answer |
| `VR_DEBUG=1 volumio-remote` | one log line per sink event (sink vs Volumio volume); stop the running instance first (single instance) |
| `~/.cache/volumio-remote.log` | stderr of autostarted instance |

Known pitfalls: see "Rules that avoid bugs" in [architecture.md](architecture.md).

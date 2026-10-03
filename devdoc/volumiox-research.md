# VolumioX research

Desktop app to control a remote Volumio instance (Linux/Windows/macOS).

| Item | Finding |
|---|---|
| Author | GitHub `majko96`; forum user `ma_sk1` |
| Source repo | https://github.com/majko96/VolumioApp (C++, Qt, qmake, no license/README details) |
| Forum thread | https://community.volumio.com/t/volumiox-small-desktop-app-for-linux-windows-and-macos/39485 |
| Binaries | Google Drive links in thread (July 2020); Linux = AppImage |
| Not related | https://github.com/lovehifi/volumiox (Volumio x64 kernel build) |
| Transport | HTTP requests to Volumio (exact endpoints undocumented) |
| Config | Volumio IP in settings dialog, no `http://` prefix (e.g. `192.168.1.10`) |

## Features

- Now-playing info (scrolling text) and volume level
- Play/Pause, Prev, Next, Mute, Volume +/-
- Arrow keys: volume +-1
- Tray icon with right-click menu, minimize to tray
- Dark, frameless window

## Source layout (VolumioApp)

| File | Role |
|---|---|
| main.cpp, mainwindow.* | main window / logic |
| scrolltext.* | scrolling text widget |
| settingsui.* | IP settings dialog |
| darkstyle.qrc, framelesswindow.qrc, res.qrc | styling/resources |
| volumio.pro | qmake project |

## Volumio REST API (known, to verify)

| Action | Request |
|---|---|
| State | `GET http://<ip>/api/v1/getState` |
| Command | `GET http://<ip>/api/v1/commands/?cmd=toggle\|play\|pause\|prev\|next` |
| Volume | `GET http://<ip>/api/v1/commands/?cmd=volume&volume=<0-100\|plus\|minus\|mute\|unmute>` |

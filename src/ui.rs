//! Slint main window. Closing hides it; tray/MPRIS can show it again.

use crate::core::Core;
use crate::volumio::{Cmd, Info};
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};
use std::process::Command;
use std::sync::{Arc, Mutex};

slint::slint! {
    import { Slider, LineEdit, Button, Palette } from "std-widgets.slint";

    component RoundButton inherits Rectangle {
        in property <length> size: 52px;
        in property <brush> fill;
        in property <bool> enabled: true;
        callback clicked();
        width: size;
        height: size;
        border-radius: size / 2;
        background: !enabled ? fill : ta.pressed ? fill.darker(0.25) : ta.has-hover ? fill.brighter(0.15) : fill;
        opacity: enabled ? 1 : 0.35;
        ta := TouchArea {
            enabled: root.enabled;
            clicked => { root.clicked(); }
        }
        @children
    }

    component Glyph inherits Path {
        in property <string> shape; // play | pause | prev | next | mute | sound
        width: 20px;
        height: 20px;
        viewbox-width: 20;
        viewbox-height: 20;
        fill: white;
        commands:
            shape == "play" ? "M 5 2 L 18 10 L 5 18 Z" :
            shape == "pause" ? "M 4 2 H 8 V 18 H 4 Z M 12 2 H 16 V 18 H 12 Z" :
            shape == "prev" ? "M 3 3 H 5.5 V 17 H 3 Z M 17 3 L 6.5 10 L 17 17 Z" :
            shape == "next" ? "M 14.5 3 H 17 V 17 H 14.5 Z M 3 3 L 13.5 10 L 3 17 Z" :
            shape == "mute" ? "M 2 7 H 6 L 11 3 V 17 L 6 13 H 2 Z M 14 7 L 19 13 M 19 7 L 14 13" :
            "M 2 7 H 6 L 11 3 V 17 L 6 13 H 2 Z M 14 7 Q 17 10 14 13";
    }

    export component MainWindow inherits Window {
        in property <bool> online;
        in property <bool> playing;
        in property <string> title-text;
        in property <string> artist-text;
        in property <string> album-text;
        in property <image> art;
        in property <bool> has-art;
        in-out property <float> volume: 50;
        in property <bool> muted;
        in property <bool> dark: true;
        in-out property <string> host;
        in-out property <bool> settings-open;
        callback toggle();
        callback prev();
        callback next();
        callback volume-released(float);
        callback toggle-mute();
        callback save-host(string);

        title: "Volume Remote";
        min-width: 320px;
        min-height: 470px;
        preferred-width: 360px;
        preferred-height: 520px;
        background: dark ? #14161b : #f3f4f7;
        changed dark => { Palette.color-scheme = dark ? ColorScheme.dark : ColorScheme.light; }
        init => { Palette.color-scheme = dark ? ColorScheme.dark : ColorScheme.light; }

        property <color> fg: dark ? #f2f3f5 : #14161b;
        property <color> sub: dark ? #9aa0ab : #5d6470;
        property <color> card: dark ? #1f222a : #ffffff;
        property <color> accent: rgb(46, 125, 255);

        VerticalLayout {
            padding: 20px;
            spacing: 16px;

            HorizontalLayout {
                spacing: 8px;
                Rectangle {
                    width: 10px;
                    height: 10px;
                    y: (parent.height - self.height) / 2;
                    border-radius: 5px;
                    background: online ? rgb(62, 207, 110) : #8a8f98;
                }
                Text {
                    text: online ? "Connected" : "Volumio offline";
                    color: sub;
                    font-size: 13px;
                    vertical-alignment: center;
                }
                Rectangle { horizontal-stretch: 1; }
                Text {
                    text: "⚙";
                    color: sub;
                    font-size: 18px;
                    TouchArea { clicked => { settings-open = !settings-open; } }
                }
            }

            Rectangle {
                background: card;
                border-radius: 16px;
                vertical-stretch: 1;
                VerticalLayout {
                    padding: 20px;
                    spacing: 6px;
                    alignment: center;
                    if has-art: HorizontalLayout {
                        alignment: center;
                        padding-bottom: 10px;
                        Rectangle {
                            width: 180px;
                            height: 180px;
                            border-radius: 12px;
                            clip: true;
                            Image { source: art; width: 100%; height: 100%; image-fit: cover; }
                        }
                    }
                    Text {
                        text: online ? (title-text != "" ? title-text : "Nothing playing") : "—";
                        color: fg;
                        font-size: 20px;
                        font-weight: 700;
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                    Text {
                        text: artist-text;
                        color: sub;
                        font-size: 15px;
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                    Text {
                        text: album-text;
                        color: sub;
                        font-size: 12px;
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                }
            }

            HorizontalLayout {
                alignment: center;
                spacing: 18px;
                RoundButton {
                    size: 46px;
                    fill: dark ? #2b2f39 : #dfe2e8;
                    enabled: online;
                    clicked => { prev(); }
                    Glyph { shape: "prev"; fill: fg; x: 13px; y: 13px; width: 20px; height: 20px; }
                }
                RoundButton {
                    size: 64px;
                    fill: accent;
                    enabled: online;
                    clicked => { toggle(); }
                    Glyph { shape: playing ? "pause" : "play"; x: 20px; y: 20px; width: 24px; height: 24px; }
                }
                RoundButton {
                    size: 46px;
                    fill: dark ? #2b2f39 : #dfe2e8;
                    enabled: online;
                    clicked => { next(); }
                    Glyph { shape: "next"; fill: fg; x: 13px; y: 13px; width: 20px; height: 20px; }
                }
            }

            HorizontalLayout {
                spacing: 12px;
                RoundButton {
                    size: 36px;
                    fill: transparent;
                    enabled: online;
                    clicked => { toggle-mute(); }
                    Glyph { shape: muted ? "mute" : "sound"; fill: transparent; stroke: sub; stroke-width: 1.6px; x: 8px; y: 8px; width: 20px; height: 20px; }
                }
                Slider {
                    minimum: 0;
                    maximum: 100;
                    value <=> root.volume;
                    enabled: online;
                    released(v) => { volume-released(v); }
                }
                Text {
                    text: round(volume);
                    color: sub;
                    font-size: 13px;
                    width: 28px;
                    horizontal-alignment: right;
                    vertical-alignment: center;
                }
            }

            if settings-open: HorizontalLayout {
                spacing: 8px;
                host-edit := LineEdit {
                    text: host;
                    placeholder-text: "Volumio host or IP, e.g. 192.168.1.10";
                    horizontal-stretch: 1;
                }
                Button {
                    text: "Save";
                    clicked => { save-host(host-edit.text); }
                }
            }
        }
    }
}

/// Dark if the Cinnamon/GNOME GTK theme name contains "dark"; default dark.
fn detect_dark(theme: &str) -> bool {
    match theme {
        "dark" => true,
        "light" => false,
        _ => ["org.cinnamon.desktop.interface", "org.gnome.desktop.interface"]
            .iter()
            .find_map(|s| {
                let o = Command::new("gsettings").args(["get", s, "gtk-theme"]).output().ok()?;
                o.status.success().then(|| String::from_utf8_lossy(&o.stdout).to_lowercase())
            })
            .map(|t| t.contains("dark"))
            .unwrap_or(true),
    }
}

fn apply(w: &MainWindow, s: &Option<Info>) {
    w.set_online(s.is_some());
    let i = s.clone().unwrap_or_default();
    w.set_playing(i.playing());
    w.set_title_text(i.title.into());
    w.set_artist_text(i.artist.into());
    w.set_album_text(i.album.into());
    w.set_muted(i.mute);
    if s.is_some() {
        w.set_volume(i.volume as f32);
    }
}

/// Download + decode cover art off-thread, then set it on the UI thread.
fn fetch_art(host: String, art: String, weak: slint::Weak<MainWindow>, current: Arc<Mutex<String>>) {
    std::thread::spawn(move || {
        let img = if art.is_empty() {
            None
        } else {
            match crate::volumio::fetch_art(&host, &art) {
                Ok(i) => Some(i),
                Err(e) => {
                    eprintln!("volumio-remote: cover art: {e}");
                    None
                }
            }
        };
        if *current.lock().unwrap() != art {
            return; // track changed meanwhile
        }
        let _ = slint::invoke_from_event_loop(move || {
            let Some(w) = weak.upgrade() else { return };
            match img {
                Some((wd, ht, px)) => {
                    w.set_art(Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&px, wd, ht)));
                    w.set_has_art(true);
                }
                None => w.set_has_art(false),
            }
        });
    });
}

pub struct Ui {
    win: MainWindow,
}

impl Ui {
    pub fn new(core: Arc<Core>) -> Ui {
        let win = MainWindow::new().expect("create window");
        win.set_dark(detect_dark(&core.cfg.lock().unwrap().theme));
        win.set_host(core.host().into());
        apply(&win, &core.snapshot());

        let c = core.clone();
        win.on_toggle(move || c.toggle());
        let c = core.clone();
        win.on_prev(move || c.run_cmd(Cmd::Prev));
        let c = core.clone();
        win.on_next(move || c.run_cmd(Cmd::Next));
        let c = core.clone();
        win.on_volume_released(move |v| c.run_cmd(Cmd::Volume(v.round() as u8)));
        let c = core.clone();
        win.on_toggle_mute(move || {
            let muted = c.snapshot().is_some_and(|i| i.mute);
            c.run_cmd(Cmd::Mute(!muted));
        });
        let c = core.clone();
        win.on_save_host(move |h| c.set_host(h.as_str()));

        let weak = win.as_weak();
        let last_art = Arc::new(Mutex::new(String::new()));
        let c = core.clone();
        core.on_change(move |s| {
            let art = s.as_ref().map(|i| i.albumart.clone()).unwrap_or_default();
            let changed = std::mem::replace(&mut *last_art.lock().unwrap(), art.clone()) != art;
            if changed {
                fetch_art(c.host(), art, weak.clone(), last_art.clone());
            }
            let s = s.clone();
            let weak = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak.upgrade() {
                    apply(&w, &s);
                }
            });
        });
        Ui { win }
    }

    /// Callable from any thread.
    pub fn show_fn(&self) -> Arc<dyn Fn() + Send + Sync> {
        let weak = self.win.as_weak();
        Arc::new(move || {
            let weak = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak.upgrade() {
                    let _ = w.show();
                }
            });
        })
    }

    pub fn run(&self, show: bool) {
        if show {
            let _ = self.win.show();
        }
        let _ = slint::run_event_loop_until_quit();
    }
}

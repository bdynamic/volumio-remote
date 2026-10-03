//! StatusNotifierItem tray (Cinnamon: xapp-sn-watcher). Offline: gray or hidden (F-31, F-34).

use crate::core::Core;
use crate::sink;
use crate::volumio::{Cmd, Info};
use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, Status};
use std::sync::Arc;
use std::time::Duration;

pub type ShowFn = Arc<dyn Fn() + Send + Sync>;

struct VrTray {
    core: Arc<Core>,
    show: ShowFn,
    info: Option<Info>,
}

const SIZE: i32 = 32;

/// ARGB32 icon: round badge + play/pause glyph. Offline: gray, translucent.
fn pixmap(online: bool, playing: bool) -> Icon {
    let (bg, fg): ([u8; 4], [u8; 4]) =
        if online { ([255, 46, 125, 255], [255, 255, 255, 255]) } else { ([150, 128, 128, 128], [150, 230, 230, 230]) };
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let c = SIZE as f32 / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let inside = (fx - c).powi(2) + (fy - c).powi(2) <= (c - 1.0).powi(2);
            let glyph = if playing {
                // pause bars
                (9.0..=13.5).contains(&fx) && (9.0..=23.0).contains(&fy) || (18.5..=23.0).contains(&fx) && (9.0..=23.0).contains(&fy)
            } else {
                // play triangle: x in 11..23, narrowing with distance from 11
                let t = (fx - 11.0) / 12.0;
                (0.0..=1.0).contains(&t) && (fy - c).abs() <= 8.0 * (1.0 - t)
            };
            data.extend_from_slice(&if !inside { [0, 0, 0, 0] } else if glyph { fg } else { bg });
        }
    }
    Icon { width: SIZE, height: SIZE, data }
}

impl ksni::Tray for VrTray {
    fn id(&self) -> String {
        "volumio-remote".into()
    }
    fn title(&self) -> String {
        match &self.info {
            Some(i) if !i.title.is_empty() => format!("{} - {}", i.artist, i.title),
            Some(_) => "Volumio Remote".into(),
            None => "Volumio offline".into(),
        }
    }
    fn status(&self) -> Status {
        if self.info.is_none() && self.core.cfg.lock().unwrap().offline_tray == "hide" { Status::Passive } else { Status::Active }
    }
    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![pixmap(self.info.is_some(), self.info.as_ref().is_some_and(|i| i.playing()))]
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        (self.show)();
    }
    fn menu(&self) -> Vec<MenuItem<Self>> {
        let online = self.info.is_some();
        let playing = self.info.as_ref().is_some_and(|i| i.playing());
        let cmd = |label: &str, c: Cmd| -> MenuItem<Self> {
            StandardItem {
                label: label.into(),
                enabled: online,
                activate: Box::new(move |t: &mut Self| t.core.run_cmd(c)),
                ..Default::default()
            }
            .into()
        };
        let mut m = vec![
            cmd(if playing { "Pause" } else { "Play" }, Cmd::Toggle),
            cmd("Previous", Cmd::Prev),
            cmd("Next", Cmd::Next),
            MenuItem::Separator,
            StandardItem {
                label: "Show window".into(),
                activate: Box::new(|t: &mut Self| (t.show)()),
                ..Default::default()
            }
            .into(),
        ];
        if sink::available() {
            m.push(
                StandardItem {
                    label: "Use as default output (volume knob)".into(),
                    activate: Box::new(|_| sink::set_default()),
                    ..Default::default()
                }
                .into(),
            );
        }
        m.push(MenuItem::Separator);
        m.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        );
        m
    }
}

pub fn spawn(core: Arc<Core>, show: ShowFn) {
    std::thread::spawn(move || {
        let make = || VrTray {
            core: core.clone(),
            show: show.clone(),
            info: core.snapshot(),
        };
        // Tray host may not be up at login: retry.
        let handle: Handle<VrTray> = loop {
            match make().spawn() {
                Ok(h) => break h,
                Err(e) => {
                    eprintln!("volumio-remote: tray register failed: {e}; retry in 3s");
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
        };
        let h = handle.clone();
        core.on_change(move |s| {
            let s = s.clone();
            h.update(move |t| t.info = s);
        });
        // Keep thread (and tray) alive; ksni runs its own runtime.
        while !handle.is_closed() {
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

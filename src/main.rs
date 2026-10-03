//! volumio-remote: Linux tray + window remote for Volumio. Spec: devdoc/requirements.md.

mod config;
mod core;
mod mpris;
mod sink;
mod tray;
mod ui;
mod volumio;

use std::path::PathBuf;
use volumio::Cmd;

/// CLI fallback (option C): bind to desktop shortcuts. Returns true if handled.
fn cli(args: &[String]) -> bool {
    // CLI mode reads the host from the config, no GUI is started
    let host = config::load().host;
    // volume +/- `d` percent relative to the current Volumio volume
    let step = |d: i16| match volumio::get_state(&host) {
        Ok(i) => volumio::send(&host, Cmd::Volume((i.volume as i16 + d).clamp(0, 100) as u8)),
        Err(e) => Err(e),
    };
    let r = match args.first().map(String::as_str) {
        Some("--toggle") => volumio::send(&host, Cmd::Toggle),
        Some("--next") => volumio::send(&host, Cmd::Next),
        Some("--prev") => volumio::send(&host, Cmd::Prev),
        Some("--vol-up") => step(5),
        Some("--vol-down") => step(-5),
        Some("--diagnose-knob") => {
            sink::diagnose();
            return true;
        }
        Some("--mute") => volumio::send(&host, Cmd::Mute(true)),
        Some("--help") | Some("-h") => {
            println!("volumio-remote [--show] | --toggle | --next | --prev | --vol-up | --vol-down | --mute");
            return true;
        }
        _ => return false,
    };
    if let Err(e) = r {
        eprintln!("volumio-remote: {e}");
        std::process::exit(1);
    }
    true
}

/// Entry: CLI command (then exit) or tray + window app.
fn main() {
    // skip program name
    let args: Vec<String> = std::env::args().skip(1).collect();
    if cli(&args) {
        return;
    }

    // Single instance; lock lives as long as the process.
    let lock_path = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("volumio-remote.lock");
    let lock = std::fs::OpenOptions::new().create(true).write(true).truncate(false).open(&lock_path).expect("open lock file");
    if lock.try_lock().is_err() {
        eprintln!("volumio-remote: already running");
        return;
    }

    // no config file yet: open the window so the host can be entered
    let first_run = !config::path().exists();
    let core = core::Core::new(config::load());
    let ui = ui::Ui::new(core.clone());
    // thread-safe "show window" callback for tray and MPRIS
    let show = ui.show_fn();
    // start background workers: polling, tray, MPRIS, volume-knob sink
    core.start_polling();
    tray::spawn(core.clone(), show.clone());
    mpris::spawn(core.clone(), show);
    sink::start(core.clone());
    // blocks in the Slint event loop until quit
    ui.run(first_run || args.iter().any(|a| a == "--show"));
    drop(lock);
}

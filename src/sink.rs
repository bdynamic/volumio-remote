//! Virtual PulseAudio/PipeWire sink: volume knob -> Volumio volume (F-28, option A).
//! User selects sink "volumio_remote" as default output; its volume/mute is mirrored.

use crate::core::Core;
use crate::volumio::Cmd;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use std::time::Duration;

/// Name of the virtual null sink the user selects as default output.
pub const SINK: &str = "volumio_remote";

/// Time of the last sink event (ms since first use). Volumio -> sink sync pauses
/// while the knob is being turned, else an intermediate Volumio value fights the user.
static LAST_KNOB_MS: AtomicU64 = AtomicU64::new(0);
static T0: OnceLock<Instant> = OnceLock::new();

/// Monotonic ms; offset 10 s so the initial 0 in `LAST_KNOB_MS` counts as "long ago".
fn now_ms() -> u64 {
    T0.get_or_init(Instant::now).elapsed().as_millis() as u64 + 10_000
}

/// Run `pactl`, return stdout on success.
fn pactl(args: &[&str]) -> Option<String> {
    // pactl output is localized ("Stumm: ja"): force C locale for parsing.
    let o = Command::new("pactl").args(args).env("LC_ALL", "C").env("LANGUAGE", "C").output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// True if `pactl` works (PulseAudio or PipeWire-pulse running).
pub fn available() -> bool {
    pactl(&["info"]).is_some()
}

/// Creates the sink if missing; false if it could not be created.
fn ensure() -> bool {
    // already listed by name (column 2 of `list short sinks`)?
    if pactl(&["list", "short", "sinks"]).is_some_and(|s| s.lines().any(|l| l.split_whitespace().nth(1) == Some(SINK))) {
        return true;
    }
    pactl(&[
        "load-module",
        "module-null-sink",
        &format!("sink_name={SINK}"),
        "sink_properties=device.description=Volumio_Remote",
    ])
    .is_some()
}

/// Make the virtual sink the default output (tray menu entry).
pub fn set_default() {
    pactl(&["set-default-sink", SINK]);
}

/// First `NN%` in `pactl get-sink-volume` output.
pub fn parse_volume(out: &str) -> Option<u8> {
    out.split_whitespace().find_map(|t| t.strip_suffix('%')?.parse::<u32>().ok()).map(|v| v.min(100) as u8)
}

/// Parse `Mute: yes|no` from `pactl get-sink-mute` (C locale).
pub fn parse_mute(out: &str) -> Option<bool> {
    match out.split(':').nth(1)?.trim() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Language independent: `pactl --format=json list sinks` (PulseAudio/PipeWire >= 16).
/// Volume (percent of first channel) and mute of our sink from the JSON listing.
pub fn parse_sink_json(text: &str) -> Option<(u8, bool)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let sink = v.as_array()?.iter().find(|s| s["name"] == SINK)?;
    let pct = sink["volume"].as_object()?.values().next()?["value_percent"].as_str()?;
    Some((pct.trim_end_matches('%').parse::<u32>().ok()?.min(100) as u8, sink["mute"].as_bool()?))
}

/// Current (volume, mute) of the virtual sink, JSON first, text as fallback.
fn read_sink() -> Option<(u8, bool)> {
    if let Some(r) = pactl(&["--format=json", "list", "sinks"]).and_then(|t| parse_sink_json(&t)) {
        return Some(r);
    }
    // Fallback: text output (forced C locale).
    let vol = parse_volume(&pactl(&["get-sink-volume", SINK])?)?;
    let mute = pactl(&["get-sink-mute", SINK]).and_then(|m| parse_mute(&m)).unwrap_or(false);
    Some((vol, mute))
}

/// Verbose logging when env `VR_DEBUG` is set.
fn debug() -> bool {
    std::env::var_os("VR_DEBUG").is_some()
}

/// `volumio-remote --diagnose-knob`: prints what the knob path sees for 15 s.
pub fn diagnose() {
    println!("pactl available: {}", available());
    println!("{}", pactl(&["info"]).unwrap_or_default().lines().filter(|l| l.contains("Server Name") || l.contains("Default Sink")).collect::<Vec<_>>().join("\n"));
    println!("sinks:\n{}", pactl(&["list", "short", "sinks"]).unwrap_or_default());
    println!("sink {SINK} exists: {}", pactl(&["list", "short", "sinks"]).is_some_and(|s| s.lines().any(|l| l.split_whitespace().nth(1) == Some(SINK))));
    println!("sink {SINK} now: {:?}", read_sink());
    println!("config host: {}", crate::config::load().host);
    println!("Turn the volume knob now (15 s). Events (this run sends real Volume commands):");
    let Ok(mut child) = Command::new("timeout").args(["15", "pactl", "subscribe"]).env("LC_ALL", "C").stdout(Stdio::piped()).spawn() else { return };
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if line.contains("sink #") || line.contains("server") {
                let sink = read_sink();
                println!("{line}  -> {SINK}: {sink:?}");
                // Same steps as the app: compare with Volumio, send, re-read.
                let host = crate::config::load().host;
                match (sink, crate::volumio::get_state(&host)) {
                    (Some((v, _)), Ok(i)) if v != i.volume => {
                        let r = crate::volumio::send(&host, Cmd::Volume(v));
                        let after = crate::volumio::get_state(&host).map(|i| i.volume);
                        println!("    {host}: volumio={} -> send Volume({v}): {r:?}, volumio now {after:?}", i.volume);
                    }
                    (Some((v, _)), Ok(i)) => println!("    {host}: volumio={} equals sink {v}, nothing to send", i.volume),
                    (_, Err(e)) => println!("    {host}: getState failed: {e}"),
                    (None, _) => println!("    cannot read sink"),
                }
            }
        }
    }
}

/// Start both sync directions between the virtual sink and Volumio.
pub fn start(core: Arc<Core>) {
    if !available() || !ensure() {
        eprintln!("volumio-remote: pactl/sink unavailable, volume knob disabled");
        return;
    }
    // Volumio -> sink (keeps knob position in sync; no-op when equal).
    // Also run once now: a new sink starts at 100%, so the knob could not go up.
    // pause while the knob moves, set sink volume only if it differs
    fn sync(s: &Option<crate::volumio::Info>) {
        if now_ms().saturating_sub(LAST_KNOB_MS.load(Ordering::SeqCst)) < 1500 {
            return;
        }
        if let Some(i) = s {
            if read_sink().is_some_and(|(v, _)| v != i.volume) {
                pactl(&["set-sink-volume", SINK, &format!("{}%", i.volume)]);
            }
        }
    }
    // listener runs on every Volumio state change
    core.on_change(sync);
    sync(&core.snapshot());
    // sink -> Volumio: react to every sink event from `pactl subscribe`
    // reconnect loop: `pactl subscribe` ends if the audio server restarts
    std::thread::spawn(move || loop {
        // C locale: event lines are translated otherwise ("auf Sink #"), the filter below would miss them.
        let child = Command::new("pactl")
            .arg("subscribe")
            .env("LC_ALL", "C")
            .env("LANGUAGE", "C")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut child) = child {
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    // "Event 'change' on sink #5": only "Event"/"on" are translated, "sink #" is not.
                    if !line.contains("sink #") {
                        continue;
                    }
                    // remember knob activity so `sync` pauses
                    LAST_KNOB_MS.store(now_ms(), Ordering::SeqCst);
                    let (Some((vol, mute)), Some(info)) = (read_sink(), core.snapshot()) else { continue };
                    if debug() {
                        eprintln!("volumio-remote: sink vol={vol} mute={mute}, volumio vol={} mute={}", info.volume, info.mute);
                    }
                    // forward mute and volume changes; volume ignored while muted
                    if mute != info.mute {
                        core.run_cmd(Cmd::Mute(mute));
                    }
                    if vol != info.volume && !mute {
                        core.run_cmd(Cmd::Volume(vol));
                    }
                }
            }
            // subscribe ended: reap the child, wait, then reconnect
            let _ = child.wait();
        }
        std::thread::sleep(Duration::from_secs(2));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json() {
        let j = r#"[{"name":"other","mute":true,"volume":{"mono":{"value_percent":"9%"}}},
                    {"name":"volumio_remote","mute":false,"volume":{"front-left":{"value":32768,"value_percent":"50%"},"front-right":{"value":32768,"value_percent":"50%"}}}]"#;
        assert_eq!(parse_sink_json(j), Some((50, false)));
        assert_eq!(parse_sink_json("[]"), None);
    }

    #[test]
    fn parses_pactl() {
        let v = "Volume: front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB\n        balance 0.00";
        assert_eq!(parse_volume(v), Some(50));
        assert_eq!(parse_volume("nothing"), None);
        assert_eq!(parse_mute("Mute: yes\n"), Some(true));
        assert_eq!(parse_mute("Mute: no\n"), Some(false));
    }
}

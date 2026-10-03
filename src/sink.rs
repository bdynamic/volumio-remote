//! Virtual PulseAudio/PipeWire sink: volume knob -> Volumio volume (F-28, option A).
//! User selects sink "volumio_remote" as default output; its volume/mute is mirrored.

use crate::core::Core;
use crate::volumio::Cmd;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

pub const SINK: &str = "volumio_remote";

fn pactl(args: &[&str]) -> Option<String> {
    // pactl output is localized ("Stumm: ja"): force C locale for parsing.
    let o = Command::new("pactl").args(args).env("LC_ALL", "C").env("LANGUAGE", "C").output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

pub fn available() -> bool {
    pactl(&["info"]).is_some()
}

/// Creates the sink if missing.
fn ensure() -> bool {
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

pub fn set_default() {
    pactl(&["set-default-sink", SINK]);
}

/// First `NN%` in `pactl get-sink-volume` output.
pub fn parse_volume(out: &str) -> Option<u8> {
    out.split_whitespace().find_map(|t| t.strip_suffix('%')?.parse::<u32>().ok()).map(|v| v.min(100) as u8)
}

pub fn parse_mute(out: &str) -> Option<bool> {
    match out.split(':').nth(1)?.trim() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

fn read_sink() -> Option<(u8, bool)> {
    let vol = parse_volume(&pactl(&["get-sink-volume", SINK])?)?;
    let mute = pactl(&["get-sink-mute", SINK]).and_then(|m| parse_mute(&m)).unwrap_or(false);
    Some((vol, mute))
}

fn debug() -> bool {
    std::env::var_os("VR_DEBUG").is_some()
}

pub fn start(core: Arc<Core>) {
    if !available() || !ensure() {
        eprintln!("volumio-remote: pactl/sink unavailable, volume knob disabled");
        return;
    }
    // Volumio -> sink (keeps knob position in sync; no-op when equal).
    core.on_change(|s| {
        if let Some(i) = s {
            if read_sink().is_some_and(|(v, _)| v != i.volume) {
                pactl(&["set-sink-volume", SINK, &format!("{}%", i.volume)]);
            }
        }
    });
    // sink -> Volumio
    std::thread::spawn(move || loop {
        let child = Command::new("pactl").arg("subscribe").stdout(Stdio::piped()).stderr(Stdio::null()).spawn();
        if let Ok(mut child) = child {
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if !line.contains(" on sink ") {
                        continue;
                    }
                    let (Some((vol, mute)), Some(info)) = (read_sink(), core.snapshot()) else { continue };
                    if debug() {
                        eprintln!("volumio-remote: sink vol={vol} mute={mute}, volumio vol={} mute={}", info.volume, info.mute);
                    }
                    if mute != info.mute {
                        core.run_cmd(Cmd::Mute(mute));
                    }
                    if vol != info.volume && !mute {
                        core.run_cmd(Cmd::Volume(vol));
                    }
                }
            }
            let _ = child.wait();
        }
        std::thread::sleep(Duration::from_secs(2));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pactl() {
        let v = "Volume: front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB\n        balance 0.00";
        assert_eq!(parse_volume(v), Some(50));
        assert_eq!(parse_volume("nothing"), None);
        assert_eq!(parse_mute("Mute: yes\n"), Some(true));
        assert_eq!(parse_mute("Mute: no\n"), Some(false));
    }
}

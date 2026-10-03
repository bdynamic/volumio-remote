//! Volumio REST client (blocking). Offline = any error from `get_state`.

use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    pub status: String, // "play" | "pause" | "stop"
    pub title: String,
    pub artist: String,
    pub album: String,
    pub volume: u8,
    pub mute: bool,
}

impl Info {
    pub fn playing(&self) -> bool {
        self.status == "play"
    }
}

pub fn parse_state(text: &str) -> Option<Info> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    Some(Info {
        status: s("status"),
        title: s("title"),
        artist: s("artist"),
        album: s("album"),
        volume: v.get("volume").and_then(|x| x.as_f64()).unwrap_or(0.0).clamp(0.0, 100.0) as u8,
        mute: v.get("mute").and_then(|x| x.as_bool()).unwrap_or(false),
    })
}

fn get(host: &str, path: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .into();
    let url = format!("http://{host}{path}");
    agent
        .get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())
}

pub fn get_state(host: &str) -> Result<Info, String> {
    parse_state(&get(host, "/api/v1/getState")?).ok_or_else(|| "bad getState reply".into())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    Toggle,
    Play,
    Pause,
    Stop,
    Prev,
    Next,
    Volume(u8),
    Mute(bool),
}

pub fn cmd_path(c: Cmd) -> String {
    let q = match c {
        Cmd::Toggle => "cmd=toggle".to_string(),
        Cmd::Play => "cmd=play".into(),
        Cmd::Pause => "cmd=pause".into(),
        Cmd::Stop => "cmd=stop".into(),
        Cmd::Prev => "cmd=prev".into(),
        Cmd::Next => "cmd=next".into(),
        Cmd::Volume(n) => format!("cmd=volume&volume={}", n.min(100)),
        Cmd::Mute(true) => "cmd=volume&volume=mute".into(),
        Cmd::Mute(false) => "cmd=volume&volume=unmute".into(),
    };
    format!("/api/v1/commands/?{q}")
}

pub fn send(host: &str, c: Cmd) -> Result<(), String> {
    get(host, &cmd_path(c)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_state() {
        let i = parse_state(r#"{"status":"play","title":"T","artist":"A","album":"B","volume":42,"mute":false,"seek":1}"#).unwrap();
        assert_eq!((i.title.as_str(), i.artist.as_str(), i.volume), ("T", "A", 42));
        assert!(i.playing());
    }

    #[test]
    fn tolerates_missing_fields() {
        let i = parse_state("{}").unwrap();
        assert_eq!(i, Info::default());
        assert!(parse_state("nope").is_none());
    }

    #[test]
    fn builds_paths() {
        assert_eq!(cmd_path(Cmd::Toggle), "/api/v1/commands/?cmd=toggle");
        assert_eq!(cmd_path(Cmd::Volume(250)), "/api/v1/commands/?cmd=volume&volume=100");
        assert_eq!(cmd_path(Cmd::Mute(true)), "/api/v1/commands/?cmd=volume&volume=mute");
    }
}

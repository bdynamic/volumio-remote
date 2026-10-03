//! Volumio REST client (blocking). Offline = any error from `get_state`.

use std::time::Duration;

/// Subset of the Volumio `getState` reply that the app uses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    pub status: String, // "play" | "pause" | "stop"
    pub title: String,
    pub artist: String,
    pub album: String,
    pub albumart: String, // path or absolute URL
    pub volume: u8,
    pub mute: bool,
}

impl Info {
    /// True while Volumio plays (not paused/stopped).
    pub fn playing(&self) -> bool {
        self.status == "play"
    }
}

/// Parse a `getState` JSON reply; missing fields get defaults. `None` if not JSON.
pub fn parse_state(text: &str) -> Option<Info> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    Some(Info {
        status: s("status"),
        title: s("title"),
        artist: s("artist"),
        album: s("album"),
        albumart: s("albumart"),
        volume: v.get("volume").and_then(|x| x.as_f64()).unwrap_or(0.0).clamp(0.0, 100.0) as u8,
        mute: v.get("mute").and_then(|x| x.as_bool()).unwrap_or(false),
    })
}

/// HTTP agent with a global timeout, so an offline host cannot block long.
fn agent(secs: u64) -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(secs))).build().into()
}

/// GET `http://<host><path>` (2 s timeout), returns the body.
fn get(host: &str, path: &str) -> Result<String, String> {
    let url = format!("http://{host}{path}");
    agent(2).get(&url).call().map_err(|e| e.to_string())?.body_mut().read_to_string().map_err(|e| e.to_string())
}

/// Absolute URL stays; Volumio paths (`/albumart?...`) get `http://<host>` prepended.
pub fn art_url(host: &str, art: &str) -> String {
    if art.starts_with("http") { art.to_string() } else { format!("http://{host}{art}") }
}

/// Downloads cover art and decodes to RGBA8 (width, height, pixels).
pub fn fetch_art(host: &str, art: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = agent(5)
        .get(&art_url(host, art))
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
    Ok((img.width(), img.height(), img.into_raw()))
}

/// Fetch current state. Any error means "offline".
pub fn get_state(host: &str) -> Result<Info, String> {
    parse_state(&get(host, "/api/v1/getState")?).ok_or_else(|| "bad getState reply".into())
}

/// Player commands sent to Volumio.
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

/// REST path for a command; volume is capped at 100.
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

/// Send a command, result body is ignored.
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
    fn builds_art_url() {
        assert_eq!(art_url("h", "/albumart?x=1"), "http://h/albumart?x=1");
        assert_eq!(art_url("h", "https://c/x.jpg"), "https://c/x.jpg");
    }

    #[test]
    fn builds_paths() {
        assert_eq!(cmd_path(Cmd::Toggle), "/api/v1/commands/?cmd=toggle");
        assert_eq!(cmd_path(Cmd::Volume(250)), "/api/v1/commands/?cmd=volume&volume=100");
        assert_eq!(cmd_path(Cmd::Mute(true)), "/api/v1/commands/?cmd=volume&volume=mute");
    }
}

//! Plain `key=value` config in `~/.config/volumio-remote/config`.

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub host: String,
    /// "gray" | "hide": tray icon while Volumio is offline (F-34)
    pub offline_tray: String,
    /// "auto" | "dark" | "light"
    pub theme: String,
    /// window opacity in percent, 30..=100
    pub opacity: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config { host: "volumio.local".into(), offline_tray: "gray".into(), theme: "auto".into(), opacity: 100 }
    }
}

pub fn path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"));
    base.join("volumio-remote/config")
}

pub fn parse(text: &str) -> Config {
    let mut c = Config::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim();
        match k.trim() {
            "host" if !v.is_empty() => c.host = v.trim_start_matches("http://").trim_end_matches('/').into(),
            "offline_tray" if v == "gray" || v == "hide" => c.offline_tray = v.into(),
            "theme" if matches!(v, "auto" | "dark" | "light") => c.theme = v.into(),
            "opacity" => {
                if let Ok(n) = v.parse::<u8>() {
                    c.opacity = n.clamp(30, 100);
                }
            }
            _ => {}
        }
    }
    c
}

pub fn load() -> Config {
    std::fs::read_to_string(path()).map(|t| parse(&t)).unwrap_or_default()
}

pub fn save(c: &Config) -> std::io::Result<()> {
    let p = path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(p, format!("host={}\noffline_tray={}\ntheme={}\nopacity={}\n", c.host, c.offline_tray, c.theme, c.opacity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_defaults() {
        assert_eq!(parse(""), Config::default());
        let c = parse("host=http://192.168.1.10/\noffline_tray=hide\ntheme=dark\nopacity=10\nbogus\n");
        assert_eq!((c.host.as_str(), c.offline_tray.as_str(), c.theme.as_str()), ("192.168.1.10", "hide", "dark"));
        assert_eq!(parse("offline_tray=x").offline_tray, "gray");
        assert_eq!((c.opacity, parse("opacity=70").opacity, parse("opacity=x").opacity), (30, 70, 100));
    }
}

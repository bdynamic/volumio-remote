//! MPRIS2 service so OS media keys / playerctl control Volumio (F-23..F-27).
//! Registered only while Volumio is online (F-32).

use crate::core::Core;
use crate::tray::ShowFn;
use crate::volumio::{Cmd, Info};
use std::collections::HashMap;
use std::sync::Arc;
use zbus::zvariant::{ObjectPath, Value};
use zbus::{connection, Connection};

/// bus name and object path required by the MPRIS2 spec
const NAME: &str = "org.mpris.MediaPlayer2.volumio_remote";
const PATH: &str = "/org/mpris/MediaPlayer2";

/// `org.mpris.MediaPlayer2`: app-level interface (raise/quit, identity).
struct Root {
    show: ShowFn,
}

#[zbus::interface(name = "org.mpris.MediaPlayer2")]
impl Root {
    fn raise(&self) {
        (self.show)();
    }
    /// quitting through MPRIS is not supported (`can_quit` = false)
    fn quit(&self) {}
    #[zbus(property)]
    fn can_quit(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn identity(&self) -> String {
        "Volumio Remote".into()
    }
    #[zbus(property)]
    fn desktop_entry(&self) -> String {
        "volumio-remote".into()
    }
    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }
    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }
}

/// `org.mpris.MediaPlayer2.Player`: transport + metadata, forwarded to Volumio.
struct Player {
    core: Arc<Core>,
}

/// Map Volumio status to the MPRIS `PlaybackStatus` string.
fn status_of(i: &Option<Info>) -> &'static str {
    match i {
        Some(i) if i.playing() => "Playing",
        Some(i) if i.status == "pause" => "Paused",
        _ => "Stopped",
    }
}

#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl Player {
    fn next(&self) {
        self.core.run_cmd(Cmd::Next);
    }
    fn previous(&self) {
        self.core.run_cmd(Cmd::Prev);
    }
    fn pause(&self) {
        self.core.run_cmd(Cmd::Pause);
    }
    fn play_pause(&self) {
        self.core.run_cmd(Cmd::Toggle);
    }
    fn stop(&self) {
        self.core.run_cmd(Cmd::Stop);
    }
    fn play(&self) {
        self.core.run_cmd(Cmd::Play);
    }
    /// seek/position/URI are not supported by this remote
    fn seek(&self, _offset: i64) {}
    fn set_position(&self, _track: ObjectPath<'_>, _pos: i64) {}
    fn open_uri(&self, _uri: String) {}

    #[zbus(property)]
    fn playback_status(&self) -> String {
        status_of(&self.core.snapshot()).into()
    }
    /// loop, rate and shuffle are fixed values; setters are no-ops
    #[zbus(property)]
    fn loop_status(&self) -> String {
        "None".into()
    }
    #[zbus(property)]
    fn set_loop_status(&self, _v: String) {}
    #[zbus(property)]
    fn rate(&self) -> f64 {
        1.0
    }
    #[zbus(property)]
    fn set_rate(&self, _v: f64) {}
    #[zbus(property)]
    fn shuffle(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn set_shuffle(&self, _v: bool) {}
    /// Track metadata; cover art is not exported yet (F-29).
    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, Value<'static>> {
        let mut m: HashMap<String, Value<'static>> = HashMap::new();
        m.insert("mpris:trackid".into(), Value::from(ObjectPath::from_static_str_unchecked("/org/volumio_remote/track")));
        if let Some(i) = self.core.snapshot() {
            m.insert("xesam:title".into(), Value::from(i.title));
            m.insert("xesam:artist".into(), Value::from(vec![i.artist]));
            m.insert("xesam:album".into(), Value::from(i.album));
        }
        m
    }
    #[zbus(property)]
    fn volume(&self) -> f64 {
        self.core.snapshot().map_or(0.0, |i| i.volume as f64 / 100.0)
    }
    #[zbus(property)]
    fn set_volume(&self, v: f64) {
        self.core.run_cmd(Cmd::Volume((v.clamp(0.0, 1.0) * 100.0).round() as u8));
    }
    /// position is not tracked
    #[zbus(property)]
    fn position(&self) -> i64 {
        0
    }
    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        1.0
    }
    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        1.0
    }
    /// capability flags: transport controls yes, seeking no
    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn can_play(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn can_pause(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn can_seek(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }
}

/// Own the bus name and export both interfaces on the session bus.
async fn connect(core: &Arc<Core>, show: &ShowFn) -> zbus::Result<Connection> {
    connection::Builder::session()?
        .name(NAME)?
        .serve_at(PATH, Root { show: show.clone() })?
        .serve_at(PATH, Player { core: core.clone() })?
        .build()
        .await
}

/// Emit PropertiesChanged so clients (playerctl, panel applets) refresh.
async fn notify(conn: &Connection) -> zbus::Result<()> {
    let r = conn.object_server().interface::<_, Player>(PATH).await?;
    let p = r.get().await;
    let ctx = r.signal_emitter();
    p.playback_status_changed(ctx).await?;
    p.metadata_changed(ctx).await?;
    p.volume_changed(ctx).await?;
    Ok(())
}

/// Run the MPRIS service in its own thread with a small tokio runtime.
/// It is registered only while Volumio is online and the `mpris` option is on.
pub fn spawn(core: Arc<Core>, show: ShowFn) {
    // listener (any thread) -> async loop: wake-up per state/option change
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    core.on_change(move |_| {
        let _ = tx.send(());
    });
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        rt.block_on(async move {
            // `Some` = registered on the bus
            let mut conn: Option<Connection> = None;
            loop {
                let online = core.snapshot().is_some() && core.cfg.lock().unwrap().mpris;
                if online && conn.is_none() {
                    match connect(&core, &show).await {
                        Ok(c) => conn = Some(c),
                        Err(e) => eprintln!("volumio-remote: MPRIS register failed: {e}"),
                    }
                } else if !online {
                    if let Some(c) = conn.take() {
                        let _ = c.graceful_shutdown().await;
                    }
                }
                if let Some(c) = &conn {
                    if let Err(e) = notify(c).await {
                        eprintln!("volumio-remote: MPRIS notify: {e}");
                    }
                }
                // wait for next state change; channel closed = shut down
                if rx.recv().await.is_none() {
                    return;
                }
            }
        });
    });
}

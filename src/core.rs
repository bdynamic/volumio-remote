//! Shared state: config, last Volumio state (None = offline), change listeners.

use crate::config::{self, Config};
use crate::volumio::{self, Cmd, Info};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Called on every state change; `None` = Volumio offline.
type Listener = Box<dyn Fn(&Option<Info>) + Send + Sync>;

/// Shared app state, used by UI, tray, MPRIS and sink threads.
pub struct Core {
    /// current settings (also persisted via `config::save`)
    pub cfg: Mutex<Config>,
    /// last known Volumio state; `None` = offline
    pub state: Mutex<Option<Info>>,
    /// UI/tray/MPRIS/sink register here via `on_change`
    listeners: Mutex<Vec<Listener>>,
    /// Bumped per command: a state fetched before a command is stale and dropped.
    cmd_seq: AtomicU64,
}

impl Core {
    /// Create core with empty state (offline until the first `refresh`).
    pub fn new(cfg: Config) -> Arc<Core> {
        Arc::new(Core { cfg: Mutex::new(cfg), state: Mutex::new(None), listeners: Mutex::new(vec![]), cmd_seq: AtomicU64::new(0) })
    }

    /// Register a listener. It runs on whichever thread triggers the change.
    pub fn on_change(&self, f: impl Fn(&Option<Info>) + Send + Sync + 'static) {
        self.listeners.lock().unwrap().push(Box::new(f));
    }

    /// Current Volumio host (cloned out of the lock).
    pub fn host(&self) -> String {
        self.cfg.lock().unwrap().host.clone()
    }

    /// Copy of the last known state.
    pub fn snapshot(&self) -> Option<Info> {
        self.state.lock().unwrap().clone()
    }

    /// Fetch state; notify listeners when it changed (online/offline included).
    pub fn refresh(&self) {
        // remember command counter before the (slow) HTTP request
        let seq = self.cmd_seq.load(Ordering::SeqCst);
        let new = volumio::get_state(&self.host()).ok();
        // a command ran meanwhile: reply is stale, drop it
        if self.cmd_seq.load(Ordering::SeqCst) != seq {
            return;
        }
        {
            let mut cur = self.state.lock().unwrap();
            // unchanged: do not wake listeners
            if *cur == new {
                return;
            }
            *cur = new.clone();
        }
        for l in self.listeners.lock().unwrap().iter() {
            l(&new);
        }
    }

    /// Send command in a worker thread, then refresh.
    pub fn run_cmd(self: &Arc<Self>, c: Cmd) {
        self.cmd_seq.fetch_add(1, Ordering::SeqCst);
        let core = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = volumio::send(&core.host(), c) {
                eprintln!("volumio-remote: {c:?}: {e}");
            }
            core.refresh();
        });
    }

    /// Play/pause toggle.
    pub fn toggle(self: &Arc<Self>) {
        self.run_cmd(Cmd::Toggle);
    }

    /// Normalize + persist a new host, then refresh in the background.
    pub fn set_host(self: &Arc<Self>, host: &str) {
        let host = host.trim().trim_start_matches("http://").trim_end_matches('/');
        // ignore empty input
        if host.is_empty() {
            return;
        }
        let cfg = {
            let mut c = self.cfg.lock().unwrap();
            c.host = host.to_string();
            c.clone()
        };
        if let Err(e) = config::save(&cfg) {
            eprintln!("volumio-remote: save config: {e}");
        }
        let core = self.clone();
        std::thread::spawn(move || core.refresh());
    }

    /// Persist one config option (`theme`, `offline_tray`) and re-notify listeners (tray status).
    pub fn set_option(&self, key: &str, val: &str) {
        let cfg = {
            let mut c = self.cfg.lock().unwrap();
            match key {
                "theme" => c.theme = val.to_string(),
                "offline_tray" => c.offline_tray = val.to_string(),
                "opacity" => c.opacity = val.parse().unwrap_or(100u8).clamp(30, 100),
                _ => return,
            }
            c.clone()
        };
        if let Err(e) = config::save(&cfg) {
            eprintln!("volumio-remote: save config: {e}");
        }
        let cur = self.snapshot();
        for l in self.listeners.lock().unwrap().iter() {
            l(&cur);
        }
    }

    /// Poll Volumio every 2 s in a background thread (no push channel is used).
    pub fn start_polling(self: &Arc<Self>) {
        let core = self.clone();
        std::thread::spawn(move || loop {
            core.refresh();
            std::thread::sleep(Duration::from_secs(2));
        });
    }
}

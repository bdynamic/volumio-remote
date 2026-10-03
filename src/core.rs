//! Shared state: config, last Volumio state (None = offline), change listeners.

use crate::config::{self, Config};
use crate::volumio::{self, Cmd, Info};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Listener = Box<dyn Fn(&Option<Info>) + Send + Sync>;

pub struct Core {
    pub cfg: Mutex<Config>,
    pub state: Mutex<Option<Info>>,
    listeners: Mutex<Vec<Listener>>,
    /// Bumped per command: a state fetched before a command is stale and dropped.
    cmd_seq: AtomicU64,
}

impl Core {
    pub fn new(cfg: Config) -> Arc<Core> {
        Arc::new(Core { cfg: Mutex::new(cfg), state: Mutex::new(None), listeners: Mutex::new(vec![]), cmd_seq: AtomicU64::new(0) })
    }

    pub fn on_change(&self, f: impl Fn(&Option<Info>) + Send + Sync + 'static) {
        self.listeners.lock().unwrap().push(Box::new(f));
    }

    pub fn host(&self) -> String {
        self.cfg.lock().unwrap().host.clone()
    }

    pub fn snapshot(&self) -> Option<Info> {
        self.state.lock().unwrap().clone()
    }

    /// Fetch state; notify listeners when it changed (online/offline included).
    pub fn refresh(&self) {
        let seq = self.cmd_seq.load(Ordering::SeqCst);
        let new = volumio::get_state(&self.host()).ok();
        if self.cmd_seq.load(Ordering::SeqCst) != seq {
            return;
        }
        {
            let mut cur = self.state.lock().unwrap();
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

    pub fn toggle(self: &Arc<Self>) {
        self.run_cmd(Cmd::Toggle);
    }

    pub fn set_host(self: &Arc<Self>, host: &str) {
        let host = host.trim().trim_start_matches("http://").trim_end_matches('/');
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

    pub fn start_polling(self: &Arc<Self>) {
        let core = self.clone();
        std::thread::spawn(move || loop {
            core.refresh();
            std::thread::sleep(Duration::from_secs(2));
        });
    }
}

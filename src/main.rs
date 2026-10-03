//! volumiox: Linux remote control for Volumio. Spec: devdoc/requirements.md.

fn main() {
    println!("volumiox {} (stub)", env!("CARGO_PKG_VERSION"));
    // Stub: keeps running so install.sh start/autostart can be tested.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

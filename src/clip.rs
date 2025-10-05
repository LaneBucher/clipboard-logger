use arboard::Clipboard;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ClipWatcher {
    last_hash: Arc<Mutex<u64>>,
}

impl ClipWatcher {
    pub fn new() -> Self {
        Self { last_hash: Arc::new(Mutex::new(0)) }
    }

    pub fn start<F>(
        &self,
        poll_ms: u64,
        max_bytes: usize,
        mut on_text: F,
    ) where
        F: FnMut(String) + Send + 'static,
    {
        let last_hash = self.last_hash.clone();
        thread::spawn(move || {
            let mut cb = Clipboard::new().ok();
            loop {
                if let Some(ref mut c) = cb {
                    if let Ok(txt) = c.get_text() {
                        let mut s = txt;
                        if s.len() > max_bytes {
                            s.truncate(max_bytes);
                            s.push_str("…");
                        }
                        let h = fxhash::hash64(s.as_bytes());
                        let mut guard = last_hash.lock();
                        if *guard != h {
                            *guard = h;
                            on_text(s);
                        }
                    }
                } else {
                    cb = Clipboard::new().ok(); // try to reconnect
                }
                thread::sleep(Duration::from_millis(poll_ms));
            }
        });
    }
}

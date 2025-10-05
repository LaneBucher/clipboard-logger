mod config;
mod logging;
mod autostart;
mod clip;

use config::{load_or_default, save, Config};
use logging::append_entry;

use eframe::egui;
use rfd::FileDialog;

use clap::Parser;
use single_instance::SingleInstance;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use parking_lot::Mutex;

/// CLI args
#[derive(Parser, Debug)]
#[command(name = "clipboard-logger")]
struct Args {
    /// Run in background (no GUI window)
    #[arg(long)]
    daemon: bool,
}

fn main() -> eframe::Result<()> {
    let args = Args::parse();

    // Headless background daemon that never opens a window
    if args.daemon {
        run_daemon();
        // If run_daemon returns, just exit cleanly
        return Ok(());
    }

    // GUI mode: ensure a daemon is up first
    let cfg = load_or_default();
    ensure_daemon_running();

    let state = AppState::new(cfg);
    let native_opts = eframe::NativeOptions::default();
    eframe::run_native(
        "Clipboard Logger",
        native_opts,
        Box::new(|_| Ok(Box::new(App { state }))),
    )
}

struct AppState {
    cfg: Config,
}

impl AppState {
    fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

struct App {
    state: AppState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clipboard Logger (GUI)");
            ui.label("Background daemon is started automatically and continues after you close this window.");

            ui.separator();
            ui.label("Output directory:");
            ui.horizontal(|ui| {
                let path_str = self.state.cfg.output_dir.to_string_lossy().to_string();
                ui.text_edit_singleline(&mut path_str.clone());
                if ui.button("Choose…").clicked() {
                    if let Some(folder) = FileDialog::new()
                        .set_directory(&self.state.cfg.output_dir)
                        .pick_folder()
                    {
                        self.state.cfg.output_dir = folder;
                        let _ = save(&self.state.cfg);
                    }
                }
                ui.label(self.state.cfg.output_dir.display().to_string());

                if ui.button("Open Folder").clicked() {
                    let _ = open_in_explorer(&self.state.cfg.output_dir);
                }
            });

            ui.separator();
            ui.checkbox(
                &mut self.state.cfg.autostart,
                "Run at login/startup (updates OS entry)",
            );
            ui.add(
                egui::Slider::new(&mut self.state.cfg.poll_interval_ms, 50..=1000)
                    .text("Poll interval (ms)"),
            );
            ui.add(
                egui::Slider::new(&mut self.state.cfg.max_log_line_bytes, 1024..=131072)
                    .text("Max entry size (bytes)"),
            );
            ui.add(
                egui::Slider::new(&mut self.state.cfg.dedupe_window, 0..=200)
                    .text("Dedupe window (entries)"),
            );

            if ui.button("Save Settings").clicked() {
                let _ = save(&self.state.cfg);
                if let Ok(exe) = std::env::current_exe() {
                    // NOTE: update autostart.rs to include "--daemon" when enabled.
                    let _ =
                        autostart::set_autostart("ClipboardLogger", &exe, self.state.cfg.autostart);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Hide Window (keep running)").clicked() {
                    // Minimize instead of closing; daemon keeps running regardless.
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }

                if ui.button("Stop Background Daemon").clicked() {
                    write_stop_file();
                }
            });

            ui.separator();
            ui.collapsing("Notes", |ui| {
                ui.label("- Each copy is appended to a daily .txt file with timestamps.");
                ui.label("- Close this window any time: the background daemon keeps logging.");
                ui.label("- Set Autostart to launch the daemon on login (requires autostart.rs to pass --daemon).");
            });
        });
    }
}

/// Spawn a detached background process that runs `--daemon`.
fn ensure_daemon_running() {
    // We rely on the daemon itself to ensure single instance.
    // Here we optimistically spawn; if one is already running, it will no-op.
    spawn_daemon();
}

fn spawn_daemon() {
    if let Ok(exe) = std::env::current_exe() {
        let mut cmd = Command::new(exe);
        cmd.arg("--daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            // DETACHED_PROCESS = 0x00000008 so GUI closing doesn't kill it
            cmd.creation_flags(0x00000008);
        }

        let _ = cmd.spawn();
    }
}

/// Headless background loop: single-instance, start worker, watch for stop-file.
fn run_daemon() {
    // Ensure only one daemon runs
    let instance = SingleInstance::new("clipboard-logger-daemon")
        .expect("failed to create single-instance guard");
    if !instance.is_single() {
        // Another daemon already running
        return;
    }

    let cfg = load_or_default();
    std::fs::create_dir_all(&cfg.output_dir).ok();

    // Start the clipboard logging worker
    start_worker(cfg.clone());

    // Watch for a stop-file written by the GUI
    use std::time::Duration;
    if let Some(dirs) = directories::ProjectDirs::from("com", "TwoDollar", "ClipboardLogger") {
        let stop_file = dirs.config_dir().join("stop-daemon");
        loop {
            if stop_file.exists() {
                let _ = std::fs::remove_file(&stop_file);
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    } else {
        // Fallback: just park forever if ProjectDirs can't resolve
        loop {
            std::thread::park();
        }
    }
}

/// Create a stop-file the daemon watches; daemon will exit when it sees this.
fn write_stop_file() {
    if let Some(dirs) = directories::ProjectDirs::from("com", "TwoDollar", "ClipboardLogger") {
        let stop_file = dirs.config_dir().join("stop-daemon");
        let _ = std::fs::create_dir_all(dirs.config_dir());
        let _ = std::fs::write(stop_file, "stop");
    }
}

/// Open a folder in the system file explorer (best-effort).
fn open_in_explorer(path: &PathBuf) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        // Try xdg-open
        Command::new("xdg-open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}

/// Start the clipboard polling + file append worker.
/// (Runs in the daemon process; GUI doesn't call this.)
fn start_worker(cfg: Config) {
    std::fs::create_dir_all(&cfg.output_dir).ok();

    let recent = Arc::new(Mutex::new(VecDeque::<u64>::new()));

    let dir: PathBuf = cfg.output_dir.clone();
    let max_bytes = cfg.max_log_line_bytes;
    let poll = cfg.poll_interval_ms;
    let dedupe_window = cfg.dedupe_window;

    let watcher = clip::ClipWatcher::new();
    watcher.start(poll, max_bytes, move |s: String| {
        if dedupe_window > 0 {
            let h = fxhash::hash64(s.as_bytes());
            let mut dq = recent.lock();
            if dq.iter().any(|&x| x == h) {
                return; // duplicate within window
            }
            dq.push_back(h);
            while dq.len() > dedupe_window {
                dq.pop_front();
            }
        }
        let _ = append_entry(&dir, &s);
    });
}

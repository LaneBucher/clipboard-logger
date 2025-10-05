use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output_dir: PathBuf,
    pub autostart: bool,
    pub poll_interval_ms: u64,
    pub max_log_line_bytes: usize,
    pub dedupe_window: usize, // number of recent entries to remember
}

impl Default for Config {
    fn default() -> Self {
        let dirs = ProjectDirs::from("com", "TwoDollar", "ClipboardLogger")
            .expect("ProjectDirs");
        let default_out = dirs.data_dir().join("logs");
        Self {
            output_dir: default_out,
            autostart: false,
            poll_interval_ms: 250,
            max_log_line_bytes: 64 * 1024, // 64KB per entry cap
            dedupe_window: 50,
        }
    }
}

pub fn config_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let dirs = ProjectDirs::from("com", "TwoDollar", "ClipboardLogger")
        .expect("ProjectDirs");
    let config_dir = dirs.config_dir().to_path_buf();
    let cfg = config_dir.join("config.toml");
    (config_dir, cfg)
}

pub fn load_or_default() -> Config {
    let (dir, file) = config_paths();
    fs::create_dir_all(&dir).ok();
    if let Ok(bytes) = fs::read(&file) {
        if let Ok(s) = String::from_utf8(bytes) {
            if let Ok(cfg) = toml::from_str::<Config>(&s) {
                return cfg;
            }
        }
    }
    let cfg = Config::default();
    save(&cfg).ok();
    cfg
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let (_, file) = config_paths();
    let s = toml::to_string_pretty(cfg).unwrap();
    fs::create_dir_all(file.parent().unwrap())?;
    fs::write(file, s)
}

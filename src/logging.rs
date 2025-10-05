use chrono::Local;
use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf};

pub fn ensure_dir(dir: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dir)
}

pub fn append_entry(dir: &PathBuf, text: &str) -> std::io::Result<()> {
    ensure_dir(dir)?;
    let day = Local::now().format("%Y-%m-%d").to_string();
    let file = dir.join(format!("{}.txt", day));
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut f = OpenOptions::new().create(true).append(true).open(file)?;
    // Replace newlines with visible separator to keep one-line-per-entry
    let sanitized = text.replace('\n', "␤");
    writeln!(f, "[{}] {}", ts, sanitized)?;
    Ok(())
}

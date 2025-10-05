#[cfg(target_os = "windows")]
pub fn set_autostart(app_name: &str, exe_path: &std::path::Path, enable: bool) -> std::io::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let (key, _) = hkcu.create_subkey(path)?;
    if enable {
        // Quote the path (spaces) and pass --daemon
        let value = format!("\"{}\" --daemon", exe_path.display());
        key.set_value(app_name, &value)?;
    } else {
        let _ = key.delete_value(app_name);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_autostart(app_name: &str, exe_path: &std::path::Path, enable: bool) -> std::io::Result<()> {
    use std::fs;

    // LaunchAgents lives in the user's Library folder
    let launch_agents = dirs_next::home_dir()
        .expect("home_dir")
        .join("Library/LaunchAgents");
    fs::create_dir_all(&launch_agents)?;
    let label = format!("com.twodollar.{}", app_name);
    let plist_path = launch_agents.join(format!("{}.plist", label));

    if enable {
        // ProgramArguments is an array; first element is executable, then args
        let contents = format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key><string>{label}</string>
    <key>ProgramArguments</key>
    <array>
      <string>{exe}</string>
      <string>--daemon</string>
    </array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
  </dict>
</plist>
"#,
            label = label,
            exe = exe_path.display()
        );
        fs::write(&plist_path, contents)?;
        // User will need to log out/in or load with `launchctl load ~/Library/LaunchAgents/{label}.plist`
    } else if plist_path.exists() {
        fs::remove_file(plist_path)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn set_autostart(app_name: &str, exe_path: &std::path::Path, enable: bool) -> std::io::Result<()> {
    use std::fs;

    let dir = dirs_next::home_dir()
        .expect("home_dir")
        .join(".config/autostart");
    fs::create_dir_all(&dir)?;
    let desktop = dir.join(format!("{}.desktop", app_name));

    if enable {
        // Exec supports quoted path plus arguments; Terminal=false for silent background start
        let contents = format!(
"[Desktop Entry]
Type=Application
Name={name}
Comment=Clipboard Logger background daemon
Exec=\"{exe}\" --daemon
Hidden=false
NoDisplay=false
Terminal=false
X-GNOME-Autostart-enabled=true
",
            name = app_name,
            exe = exe_path.display(),
        );
        fs::write(desktop, contents)?;
    } else if desktop.exists() {
        fs::remove_file(desktop)?;
    }
    Ok(())
}

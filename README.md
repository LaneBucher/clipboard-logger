# Clipboard Logger

A lightweight cross-platform clipboard history logger written in Rust.

## Overview

Clipboard Logger runs silently in the background and logs everything you copy to a text file.  
It can start automatically on system boot and includes a simple GUI for configuration.

Each clipboard entry is timestamped and saved to daily `.txt` files inside the directory you choose.

## Features

- Lightweight and memory-efficient
- Logs all clipboard copies automatically
- Simple GUI for choosing log directory and settings
- Background daemon keeps running after you close the window
- Optional autostart on system boot
- Windows installer (`.msi`) included

## Usage

After installing with the provided `.msi` file:

1. Open **Clipboard Logger** from the Start Menu.  
2. The GUI appears for the first launch. Configure settings:
   - Output directory
   - Poll interval (how often to check clipboard)
   - Entry size and deduplication window
   - Enable or disable autostart
3. Once you close the window, Clipboard Logger continues running in the background.
4. Reopen the GUI anytime to stop, change, or view settings.

Your clipboard history is stored in: Documents/ClipboardLogs/YYYY-MM-DD.txt

## Install

To build from source:

```bash
cargo build --release
```

## To create the Windows installer

```bash
cargo wix
```

## The installer (.msi) will be in:

```php-template
target/wix/clipboard-logger-<version>-x86_64.msi
```

# Uninstall
Use **Add or Remove Programs** in Windows, or run the uninstaller from the installation directory.

# License
MIT License © 2025 Lane Bucher
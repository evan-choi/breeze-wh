<div align="center">

# Breeze

**Stop clicking "OK" after Windows Hello recognizes you.**

[![CI](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml/badge.svg)](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/breeze-wh)](https://crates.io/crates/breeze-wh)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

[English](#how-it-works) | [한국어](README.ko.md)

</div>

---

Windows Hello face recognition works great — except it still asks you to click "OK" every single time. That extra click is just noise.

**Breeze** removes it. It monitors Windows Hello credential dialogs, detects successful face recognition, and auto-confirms — silently, as a Windows Service.

## How It Works

```
Windows Hello recognizes your face
        ↓
Credential dialog appears with "OK" button
        ↓
Breeze detects it via UI Automation API
        ↓
Confirms automatically (only for face recognition, not PIN)
        ↓
You're in. Zero clicks.
```

## Install

```powershell
cargo install breeze-wh
breeze-wh install
breeze-wh start
```

That's it. Breeze runs silently in the background.

## Commands

| Command | Description |
|---------|-------------|
| `breeze-wh install` | Register the Windows Service |
| `breeze-wh uninstall` | Stop and remove the service |
| `breeze-wh start` | Start the service |
| `breeze-wh stop` | Stop the service |
| `breeze-wh status` | Check service status |

## Uninstall

```powershell
breeze-wh uninstall
cargo uninstall breeze-wh
```

## How It's Built

Breeze is a single binary that runs in two modes:

- **Service mode** — Runs in Session 0 as a Windows Service. Detects user logon/logoff and spawns the helper in the user's desktop session. Automatically restarts it on crash with exponential backoff.

- **Helper mode** — Runs in the user session with elevated privileges. Subscribes to UI Automation focus events to detect `Credential Dialog Xaml Host` windows. On detection, scans the UI tree in a single pass:
  - `PasswordField` present → PIN mode → **skip**
  - `OkButton` present without `PasswordField` → face recognition → **click**
  - `OkButton` not yet visible → watch `StructureChanged` events until it appears

All detection uses language-independent `AutomationId` and `ClassName` properties, so it works regardless of your Windows display language.

## Configuration

Config file: `C:\ProgramData\Breeze-WH\config.toml` (created on install)

```toml
enabled = true
debounce_ms = 2000
log_level = "info"
log_max_files = 7
```

Logs: `C:\ProgramData\Breeze-WH\logs\`

## Requirements

- Windows 10 / 11
- Windows Hello face recognition configured
- [Rust](https://rustup.rs/) toolchain (to install via cargo)

## License

MIT OR Apache-2.0

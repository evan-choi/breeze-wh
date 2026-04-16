<div align="center">

# Breeze

**Stop clicking "OK" after Windows Hello recognizes you.**

[![CI](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml/badge.svg)](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml)
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
cargo install --git https://github.com/evan-choi/breeze-wh
```

Then register and start the service (requires Administrator):

```powershell
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

Breeze runs as a single binary with two internal modes:

- **Service mode** — Runs in Session 0 as a Windows Service. Monitors user logon/logoff and spawns the helper in the user's desktop session. Restarts it automatically if it crashes (with exponential backoff).

- **Helper mode** — Runs in the user session with administrator privileges. Subscribes to UI Automation focus events to detect `Credential Dialog Xaml Host` windows. When one appears, it scans the UI tree in a single pass:
  - If `PasswordField` is present → PIN mode → **ignore**
  - If `OkButton` is present without `PasswordField` → face recognition → **click**
  - If `OkButton` hasn't appeared yet → watch for `StructureChanged` events until it does

All detection uses language-independent `AutomationId` and `ClassName` properties, so it works regardless of your Windows display language.

## Configuration

Config file: `C:\ProgramData\Breeze\config.toml` (created automatically on install)

```toml
enabled = true
debounce_ms = 2000
log_level = "info"
log_max_files = 7
```

Logs are written to `C:\ProgramData\Breeze\logs\`.

## Requirements

- Windows 10 / 11
- Windows Hello with face recognition configured
- Rust 1.85+ (to build from source)

## License

MIT OR Apache-2.0

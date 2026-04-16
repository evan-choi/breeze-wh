# Breeze

Auto-confirm Windows Hello dialogs. Breeze runs as a Windows Service and automatically clicks the "OK" button when Windows Hello face recognition completes.

## Requirements

- Windows 10 / 11
- Windows Hello face recognition configured
- Administrator privileges (for installation)

## Installation

```powershell
# Build from source
cargo build --release

# Install the service (requires Administrator)
breeze install

# Start the service
breeze start
```

## Usage

Once installed and started, Breeze runs in the background. When a Windows Hello face recognition dialog appears and successfully recognizes you, Breeze automatically confirms it.

```powershell
breeze status     # Check service status
breeze stop       # Stop the service
breeze uninstall  # Remove the service
```

## Architecture

Breeze uses a two-process architecture:

- **breeze-service.exe** - Windows Service (Session 0). Monitors user sessions and manages the helper process.
- **breeze-helper.exe** - Runs in the user session. Uses Windows UI Automation to detect and auto-confirm credential dialogs.

## Configuration

Configuration file: `C:\ProgramData\Breeze\config.toml`

```toml
enabled = true
debounce_ms = 2000
log_level = "info"
log_max_files = 7
```

## License

MIT OR Apache-2.0

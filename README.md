# inventory-agent

Windows Service agent for endpoint inventory collection. Part of the Big Brother endpoint inventory system.

## Overview

The inventory-agent runs as a Windows Service on endpoint machines, periodically collecting system information via WMI and reporting it to the inventory-server.

### Data Collected
- Hostname
- IP address
- Currently logged-in user
- BIOS serial number (laptop serial)
- Drive information (model, serial, device ID)

## Build

On a Windows build host with Rust toolchain:

```powershell
cargo build --release
```

Output: `target\release\inventory-agent.exe`

## Configuration

Set environment variables for the service account (LocalSystem):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `INVENTORY_API_URL` | Yes | - | Server endpoint URL (e.g., `https://server:8443/checkin`) |
| `INVENTORY_INTERVAL_SECONDS` | No | 1800 | Check-in interval (30 minutes default) |
| `INVENTORY_TLS_INSECURE` | No | false | Skip TLS verification (lab environments only) |

## Installation

### Manual Installation

1. Copy `inventory-agent.exe` to `C:\Program Files\InventoryAgent\`
2. Set environment variables for the LocalSystem account
3. Register and start the service:

```powershell
sc.exe create InventoryAgent binPath= "C:\Program Files\InventoryAgent\inventory-agent.exe" start= auto
sc.exe start InventoryAgent
```

### Service Management

```powershell
# Check status
sc.exe query InventoryAgent

# Stop service
sc.exe stop InventoryAgent

# Start service
sc.exe start InventoryAgent

# Remove service
sc.exe delete InventoryAgent
```

## Development

For interactive testing during development, you can temporarily modify `main.rs` to call `collector::collect()` and `sender::send()` directly instead of using the Windows Service entry point.

## Architecture

```
src/
├── main.rs      # Service entry point
├── service.rs   # Windows Service registration and control loop
├── collector.rs # WMI queries for system data
├── sender.rs    # HTTP POST to server endpoint
├── models.rs    # CheckIn and Drive data structures
└── config.rs    # Configuration handling
```

### Data Flow
1. Service wakes up on configured interval
2. `collector.rs` queries WMI for system information
3. Data is serialized to `CheckIn` JSON structure
4. `sender.rs` POSTs to the configured API endpoint

## Platform Requirements

- Windows only (Windows Services, WMI)
- Builds must occur on Windows hosts with Rust toolchain
- Cross-compilation from Linux/macOS is not supported due to `windows-service` and `wmi` crate dependencies

## License

MIT

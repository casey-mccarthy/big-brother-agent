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

The agent reads `config.toml` from the directory containing the executable, then lets environment variables override it. On first start, if no `config.toml` exists, a commented template is written next to the executable.

```toml
api_url = "https://server:8443/checkin"
interval_seconds = 1800
tls_insecure = false
```

Environment variables (set for the LocalSystem service account) override the file:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `INVENTORY_API_URL` | Yes | - | Server endpoint URL (e.g., `https://server:8443/checkin`) |
| `INVENTORY_INTERVAL_SECONDS` | No | 1800 | Check-in interval (30 minutes default) |
| `INVENTORY_TLS_INSECURE` | No | false | Skip TLS verification (lab environments only) |

TLS certificates are verified against the Windows certificate store, so a server certificate issued by your enterprise CA is trusted without any extra configuration.

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

Two flags run the agent in the foreground instead of as a service:

```powershell
cargo run -- --test    # collect once, print the JSON, send if api_url is set, exit
cargo run -- --debug   # collect and send on the configured interval until Ctrl+C
```

### Wire contract with the server

`tests/fixtures/checkin.json` is the canonical check-in payload and is committed identically to the inventory-server repository. `tests/contract.rs` asserts this agent serializes exactly that JSON; the server's `tests/contract.rs` asserts it accepts exactly that JSON. Change the schema on both sides and update the fixture in both repos.

`cargo test` runs on Linux and macOS too: `collector` and `service` are Windows-only, but `models`, `sender`, and `config` are tested everywhere.

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

# Inventory Agent Guide

The inventory agent is a Windows Service that periodically collects hardware and user information from the endpoint and sends it to the inventory server.

## Building

```powershell
cd inventory-agent
cargo build --release
```

The compiled binary will be at `target/release/inventory-agent.exe`.

## Configuration

The agent is configured via environment variables. When running as a Windows Service, these are set in the registry by the installer.

### Environment Variables

| Variable | Required | Description | Default |
|----------|----------|-------------|---------|
| `INVENTORY_API_URL` | Yes | Server endpoint URL (e.g., `https://server:8443/checkin`) | (none) |
| `INVENTORY_INTERVAL_SECONDS` | No | Check-in interval in seconds | `1800` (30 min) |
| `INVENTORY_TLS_INSECURE` | No | Skip TLS certificate verification (lab only) | `false` |

## Running Modes

### Windows Service (Production)

The agent is designed to run as a Windows Service. Use the [deployment scripts](deployment.md) to install it.

**Service Details:**
| Property | Value |
|----------|-------|
| Service Name | `InventoryAgent` |
| Display Name | `Endpoint Inventory Agent` |
| Startup Type | Automatic |
| Log On As | LocalSystem |
| Install Location | `C:\Program Files\InventoryAgent\` |

### Test Mode (`--test`)

One-shot mode for development testing. Collects inventory once and displays it.

```powershell
cd inventory-agent

# Collect and display only
cargo run -- --test

# Collect and send to server
$env:INVENTORY_API_URL = "http://localhost:8443/checkin"
cargo run -- --test
```

**Output:**
```
=== Inventory Agent Test Mode ===

Collecting inventory...

Collected data:
{
  "hostname": "LAPTOP-ABC123",
  "ip_address": "192.168.1.100",
  "logged_in_user": "DOMAIN\\jsmith",
  "laptop_serial": "ABC123XYZ",
  "drives": [
    {
      "model": "Samsung SSD 970 EVO 500GB",
      "serial_number": "S4EVNX0M123456",
      "device_id": "\\\\.\\PHYSICALDRIVE0"
    }
  ],
  "timestamp_utc": "2024-01-15T10:30:00.123456789Z"
}

Sending to: http://localhost:8443/checkin
Send successful!
```

### Debug Mode (`--debug`)

Foreground mode with periodic check-ins. Useful for testing the full check-in cycle without installing as a service.

```powershell
cd inventory-agent

# Run with default 30-minute interval
$env:INVENTORY_API_URL = "http://localhost:8443/checkin"
cargo run -- --debug

# Run with custom interval (60 seconds)
$env:INVENTORY_API_URL = "http://localhost:8443/checkin"
$env:INVENTORY_INTERVAL_SECONDS = "60"
cargo run -- --debug
```

**Output:**
```
[DEBUG] Starting inventory agent in debug mode...
[DEBUG] Check-in interval: 60 seconds
[DEBUG] API URL: http://localhost:8443/checkin

[DEBUG] Collecting inventory...
[DEBUG] Collected data:
{
  "hostname": "LAPTOP-ABC123",
  ...
}

[DEBUG] Sending check-in...
[DEBUG] Check-in sent successfully

[DEBUG] Next check-in in 60 seconds. Press Ctrl+C to exit.
```

Press `Ctrl+C` to stop.

## Data Collected

The agent collects the following information via WMI:

| Field | WMI Source | Description |
|-------|------------|-------------|
| `hostname` | `%COMPUTERNAME%` | Windows computer name |
| `ip_address` | Network interfaces | Primary IPv4 address (non-loopback, non-APIPA) |
| `logged_in_user` | `Win32_ComputerSystem.UserName` | Currently logged-in user (DOMAIN\Username) |
| `laptop_serial` | `Win32_BIOS.SerialNumber` | BIOS/chassis serial number |
| `drives` | `Win32_DiskDrive` | List of physical drives |
| `timestamp_utc` | System clock | ISO-8601 UTC timestamp |

### Drive Information

For each physical drive:
| Field | Description |
|-------|-------------|
| `model` | Drive model name (e.g., "Samsung SSD 970 EVO 500GB") |
| `serial_number` | Drive serial number (may be null) |
| `device_id` | Windows device path (e.g., `\\.\PHYSICALDRIVE0`) |

## JSON Payload

The agent sends the following JSON structure to the server:

```json
{
  "hostname": "string",
  "ip_address": "string",
  "logged_in_user": "string|null",
  "laptop_serial": "string",
  "drives": [
    {
      "model": "string",
      "serial_number": "string|null",
      "device_id": "string"
    }
  ],
  "timestamp_utc": "ISO-8601 string"
}
```

## Troubleshooting

### Service Won't Start

1. **Check environment variables are set:**
   ```powershell
   # View service registry settings
   Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Services\InventoryAgent"
   ```

2. **Verify API URL is reachable:**
   ```powershell
   Test-NetConnection -ComputerName inventory-server -Port 8443
   ```

3. **Check Windows Event Log:**
   ```powershell
   Get-EventLog -LogName Application -Source InventoryAgent -Newest 10
   ```

### Test Mode Fails to Collect

1. **Run as Administrator** - WMI queries may require elevated privileges
2. **Check WMI service is running:**
   ```powershell
   Get-Service winmgmt
   ```

### Check-ins Not Reaching Server

1. **Test connectivity:**
   ```powershell
   Invoke-WebRequest -Uri "http://inventory-server:8443/" -Method GET
   ```

2. **Check firewall rules:**
   ```powershell
   Get-NetFirewallRule -DisplayName "*Inventory*"
   ```

3. **For self-signed certificates in lab environments:**
   ```powershell
   $env:INVENTORY_TLS_INSECURE = "true"
   ```
   > **Warning:** Never use `TLS_INSECURE` in production.

### Missing Data Fields

| Issue | Likely Cause |
|-------|--------------|
| `logged_in_user` is null | No user logged in, or WMI query failed |
| `laptop_serial` is "UNKNOWN" | BIOS doesn't expose serial, or VM environment |
| `drives` is empty | No physical drives detected, or WMI query failed |
| `ip_address` is "0.0.0.0" | No valid network interface found |

## Network Requirements

| Direction | Port | Protocol | Purpose |
|-----------|------|----------|---------|
| Outbound | 8443 (configurable) | HTTPS/HTTP | Check-in to server |

Ensure outbound access from endpoints to the inventory server on the configured port.

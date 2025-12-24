# SPEC-agent.md — Endpoint Inventory Agent (Windows Service, Rust)

## 1) Overview
The Endpoint Inventory Agent is a compiled Windows service that periodically collects endpoint inventory data and submits it over HTTPS to the central Inventory API server.

**Service name:** `InventoryAgent`  
**Display name:** `Endpoint Inventory Agent`  
**Account:** `LocalSystem`  
**Startup:** Automatic (Delayed Start optional)

## 2) Functional Requirements
### 2.1 Data to Collect
- `hostname`: Windows computer name
- `ip_address`: primary IPv4 address (non-APIPA, non-loopback, preferring “Up” interface)
- `logged_in_user`: domain\username (best-effort)
- `laptop_serial`: chassis serial number (BIOS serial)
- `drives`: list of physical disks
  - `model`
  - `serial_number`
  - `device_id` (e.g., \\.\PHYSICALDRIVE0)
- `timestamp_utc`: ISO-8601 UTC timestamp

### 2.2 Transmission
- POST JSON to: `POST https://<server>:<port>/checkin`
- Content-Type: `application/json`
- Timeouts:
  - connect: 5s
  - request: 15s
- Retries: best-effort; if failure occurs, retry at next interval (no tight loops).

### 2.3 Scheduling
- Interval loop within the service process.
- Default: 30 minutes.
- Configurable via environment variable: `INVENTORY_INTERVAL_SECONDS`.

## 3) Configuration
Configuration sources in order of precedence:
1. Environment variables
2. `C:\ProgramData\InventoryAgent\config.json` (optional future enhancement)

### Required Environment Variables
- `INVENTORY_API_URL` — e.g., `https://inventory-server.domain.local:8443/checkin`

### Optional Environment Variables
- `INVENTORY_INTERVAL_SECONDS` — default `1800`
- `INVENTORY_TLS_INSECURE` — `true/false` (default `false`). For lab/self-signed testing only.
- `INVENTORY_USER_LOOKUP_MODE` — `wmi` (default). Placeholder for future.

## 4) Logging & Observability
- Write operational logs to Windows Event Log (Application) with source: `InventoryAgent`.
- Minimum events:
  - Service start/stop
  - Successful check-in (optional info level)
  - Failure with error category:
    - collection error
    - network error
    - server error (HTTP >= 500)
    - client error (HTTP 4xx)

## 5) Security Assumptions
- Runs on internal trusted network.
- HTTPS required.
- Server firewall restricts ingress to trusted subnets.
- Future enhancements: request signing (HMAC), mTLS, agent identity.

## 6) Implementation Notes
- Windows Service implemented with `windows-service` crate.
- Collection uses WMI via `wmi` crate for:
  - `Win32_ComputerSystem` (UserName)
  - `Win32_BIOS` (SerialNumber)
  - `Win32_DiskDrive` (Model, SerialNumber, DeviceID)
- IP address collection via `GetAdaptersAddresses` (Windows API) OR fallback to Rust `ipconfig`-like enumeration.
  - This repo uses `get_if_addrs` as a pragmatic baseline; you can swap to Windows API if needed.

## 7) Acceptance Criteria
- Service installs and stays running.
- On interval, agent produces a POST request with correct JSON schema.
- Errors do not crash the service; they are logged and agent continues.

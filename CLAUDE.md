# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **inventory-agent** component of the Big Brother endpoint inventory system. It is a Windows Service that collects hardware/user inventory from endpoints and POSTs JSON payloads to the inventory-server.

## Build & Run Commands

Build:
```powershell
cargo build --release
```

The agent normally runs as a Windows Service. For interactive development use the foreground flags: `cargo run -- --test` (collect once, print, send if configured) or `cargo run -- --debug` (loop on the interval until Ctrl+C).

Tests: `cargo test` works on any platform. `collector` and `service` are compiled only on Windows; `models`, `sender`, and `config` are tested everywhere.

## Architecture

### Source Files (src/)
- **main.rs**: Service entry point
- **service.rs**: Windows Service registration and control loop (30min default interval)
- **collector.rs**: WMI queries for system data (hostname, IP, logged-in user, BIOS serial, drives)
- **sender.rs**: HTTP POST to server endpoint
- **models.rs**: CheckIn and Drive data structures
- **config.rs**: Configuration handling

### Data Flow
1. Agent collects inventory via WMI queries (collector.rs)
2. Agent serializes CheckIn struct to JSON (models.rs)
3. Agent POSTs to /checkin endpoint (sender.rs)

### Wire Contract With the Server
`tests/fixtures/checkin.json` is the canonical check-in payload and is committed identically to the inventory-server repository. `tests/contract.rs` asserts the agent serializes exactly that JSON; the server's `tests/contract.rs` asserts it accepts exactly that JSON. Change the schema on both sides and update the fixture in both repos. `timestamp_utc` is RFC 3339 with whole seconds and a `Z` suffix (`models::utc_now_rfc3339`).

### Configuration
`config.toml` next to the executable (auto-generated as a template on first run), overridden by environment variables (set for LocalSystem service account):
- `INVENTORY_API_URL` (required)
- `INVENTORY_INTERVAL_SECONDS` (optional, default 1800)
- `INVENTORY_TLS_INSECURE` (optional, lab-only flag)

TLS is verified against the Windows certificate store (reqwest `rustls-tls-native-roots`), so enterprise-CA certificates work without `INVENTORY_TLS_INSECURE`.

## Platform Requirements
- This is a Windows-only codebase (Windows Services, WMI)
- Builds must occur on Windows hosts with Rust toolchain
- Development/testing requires Windows environment for service functionality
- Cross-compilation from Linux/macOS is not supported due to windows-service and wmi crate dependencies

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **inventory-agent** component of the Big Brother endpoint inventory system. It is a Windows Service that collects hardware/user inventory from endpoints and POSTs JSON payloads to the inventory-server.

## Build & Run Commands

Build:
```powershell
cargo build --release
```

The agent must run as a Windows Service. For interactive testing during development, you can temporarily modify main.rs to call `collector::collect()` and `sender::send()` directly instead of using the service entry point.

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

### Configuration
Environment variables (set for LocalSystem service account):
- `INVENTORY_API_URL` (required)
- `INVENTORY_INTERVAL_SECONDS` (optional, default 1800)
- `INVENTORY_TLS_INSECURE` (optional, lab-only flag)

## Platform Requirements
- This is a Windows-only codebase (Windows Services, WMI)
- Builds must occur on Windows hosts with Rust toolchain
- Development/testing requires Windows environment for service functionality
- Cross-compilation from Linux/macOS is not supported due to windows-service and wmi crate dependencies

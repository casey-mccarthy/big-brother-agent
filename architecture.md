# Inventory Agent Architecture

This document provides comprehensive architectural documentation for the inventory-agent Windows Service, including UML class diagrams, entity relationships, and sequence diagrams.

## Table of Contents

1. [Overview](#overview)
2. [Module Dependency Diagram](#module-dependency-diagram)
3. [UML Class Diagrams](#uml-class-diagrams)
4. [Entity Relationship Diagram](#entity-relationship-diagram)
5. [Sequence Diagrams](#sequence-diagrams)

---

## Overview

The inventory-agent is a Windows Service written in Rust that periodically collects endpoint hardware/user inventory data via WMI queries and POSTs the collected data as JSON to a central inventory-server.

**Key Components:**
- **main.rs** - Application entry point (service, test, or debug mode)
- **service.rs** - Windows Service registration and control loop
- **config.rs** - Configuration loading from TOML and environment variables
- **collector.rs** - WMI queries for system data collection
- **sender.rs** - HTTP POST to server endpoint
- **models.rs** - Data structures for CheckIn and Drive

---

## Module Dependency Diagram

```mermaid
flowchart TB
    subgraph Entry["Entry Points"]
        main["main.rs"]
    end

    subgraph Core["Core Modules"]
        service["service.rs<br/>(Windows Service)"]
        config["config.rs<br/>(Configuration)"]
        collector["collector.rs<br/>(WMI Queries)"]
        sender["sender.rs<br/>(HTTP Client)"]
        models["models.rs<br/>(Data Structures)"]
    end

    subgraph External["External Systems"]
        WMI["Windows WMI"]
        Server["Inventory Server<br/>/checkin endpoint"]
        SCM["Windows SCM"]
        FS["File System<br/>config.toml"]
    end

    main --> service
    main --> config
    main --> collector
    main --> sender

    service --> config
    service --> collector
    service --> sender
    service --> SCM

    collector --> models
    collector --> WMI
    sender --> models
    sender --> Server
    config --> FS
```

---

## UML Class Diagrams

### Data Models

```mermaid
classDiagram
    class CheckIn {
        +String hostname
        +String ip_address
        +Option~String~ logged_in_user
        +String laptop_serial
        +Vec~Drive~ drives
        +String timestamp_utc
    }

    class Drive {
        +String model
        +Option~String~ serial_number
        +String device_id
    }

    class Config {
        +Option~String~ api_url
        +u64 interval_seconds
        +bool tls_insecure
        +default() Config
    }

    CheckIn "1" *-- "0..*" Drive : contains

    note for CheckIn "Derives: Debug, Serialize, Deserialize, Clone\nJSON payload sent to /checkin endpoint"
    note for Drive "Derives: Debug, Serialize, Deserialize, Clone\nRepresents a physical disk drive"
    note for Config "Derives: Debug, Deserialize, Clone\nImplements: Default trait\nDefault interval: 1800 seconds"
```

### Internal WMI Structs (collector.rs)

```mermaid
classDiagram
    class Win32ComputerSystem {
        <<internal>>
        +Option~String~ user_name
    }

    class Win32Bios {
        <<internal>>
        +Option~String~ serial_number
    }

    class Win32DiskDrive {
        <<internal>>
        +Option~String~ model
        +Option~String~ serial_number
        +Option~String~ device_id
    }

    note for Win32ComputerSystem "WMI Query: SELECT UserName FROM Win32_ComputerSystem\nSerde rename: UserName"
    note for Win32Bios "WMI Query: SELECT SerialNumber FROM Win32_BIOS\nSerde rename: SerialNumber"
    note for Win32DiskDrive "WMI Query: SELECT Model, SerialNumber, DeviceID FROM Win32_DiskDrive\nSerde renames for PascalCase WMI fields"
```

### Module Functions

```mermaid
classDiagram
    class main {
        +main() Result~()~
        +run_test_mode() Result~()~
        +run_debug_mode() Result~()~
    }

    class config {
        +load_config() Result~Config~
        +exe_dir() Result~PathBuf~
        -default_interval() u64
        -generate_template_config(path: PathBuf) Result~()~
    }

    class collector {
        +collect() Result~CheckIn~
        -primary_ipv4() Option~String~
    }

    class sender {
        +send(checkin: CheckIn, api_url: str, tls_insecure: bool) Result~()~$
    }

    class service {
        +run() Result~()~
        -service_main(arguments: Vec~OsString~)
        -run_service() Result~()~
    }

    note for sender "$ = async function\nTimeout: 5s connect, 15s total"
    note for service "SERVICE_NAME = 'InventoryAgent'\nUses windows_service crate"
    note for collector "Uses wmi crate for WMI queries\nUses get_if_addrs for IP detection"
```

---

## Entity Relationship Diagram

```mermaid
erDiagram
    CONFIG ||--o{ SERVICE : "configures"
    CONFIG {
        string api_url "Optional API endpoint URL"
        int interval_seconds "Check-in interval (default 1800)"
        boolean tls_insecure "Accept invalid TLS certs"
    }

    CHECKIN ||--|{ DRIVE : "contains"
    CHECKIN {
        string hostname "Computer name"
        string ip_address "Primary IPv4 address"
        string logged_in_user "Optional current user"
        string laptop_serial "BIOS serial number"
        string timestamp_utc "RFC3339 timestamp"
    }

    DRIVE {
        string model "Drive model name"
        string serial_number "Optional drive serial"
        string device_id "Physical drive ID"
    }

    SERVICE ||--o{ CHECKIN : "produces"
    SERVICE {
        string name "InventoryAgent"
        enum status "Running, Stopped"
    }
```

---

## Sequence Diagrams

### Windows Service Execution Flow

```mermaid
sequenceDiagram
    autonumber
    participant SCM as Windows SCM
    participant Main as main.rs
    participant Service as service.rs
    participant Config as config.rs
    participant Collector as collector.rs
    participant WMI as Windows WMI
    participant Sender as sender.rs
    participant Server as Inventory Server

    SCM->>Main: Start service.exe
    Main->>Service: run()
    Service->>Service: Register service dispatcher
    Service->>Config: load_config()
    Config->>Config: Read config.toml
    Config->>Config: Apply env var overrides
    Config-->>Service: Config

    Service->>SCM: Set status: Running

    loop Every interval_seconds (default 30 min)
        Service->>Service: Check shutdown signal
        alt Shutdown requested
            Service->>SCM: Set status: Stopped
        else Continue
            Service->>Collector: collect()
            Collector->>Collector: Get COMPUTERNAME env var
            Collector->>Collector: primary_ipv4()
            Collector->>WMI: SELECT UserName FROM Win32_ComputerSystem
            WMI-->>Collector: logged_in_user
            Collector->>WMI: SELECT SerialNumber FROM Win32_BIOS
            WMI-->>Collector: laptop_serial
            Collector->>WMI: SELECT Model,SerialNumber,DeviceID FROM Win32_DiskDrive
            WMI-->>Collector: Vec<Drive>
            Collector-->>Service: CheckIn

            Service->>Sender: send(checkin, url, tls_insecure)
            Sender->>Server: POST /checkin (JSON)
            Server-->>Sender: HTTP 200 OK
            Sender-->>Service: Ok(())

            Service->>Service: Sleep interval_seconds
        end
    end
```

### Test Mode Execution Flow (--test)

```mermaid
sequenceDiagram
    autonumber
    participant CLI as Command Line
    participant Main as main.rs
    participant Config as config.rs
    participant Collector as collector.rs
    participant WMI as Windows WMI
    participant Sender as sender.rs
    participant Server as Inventory Server
    participant Console as stdout

    CLI->>Main: cargo run -- --test
    Main->>Main: Detect --test flag
    Main->>Config: load_config()
    Config-->>Main: Config

    Main->>Collector: collect()
    Collector->>WMI: WMI Queries
    WMI-->>Collector: System data
    Collector-->>Main: CheckIn

    Main->>Console: Print JSON (serde_json::to_string_pretty)

    alt INVENTORY_API_URL is set
        Main->>Sender: send(checkin, url, tls_insecure)
        Sender->>Server: POST /checkin (JSON)
        Server-->>Sender: HTTP Response
        Sender-->>Main: Result
    end

    Main->>CLI: Exit
```

### Debug Mode Execution Flow (--debug)

```mermaid
sequenceDiagram
    autonumber
    participant CLI as Command Line
    participant Main as main.rs
    participant Config as config.rs
    participant Collector as collector.rs
    participant WMI as Windows WMI
    participant Sender as sender.rs
    participant Server as Inventory Server
    participant Console as stdout

    CLI->>Main: cargo run -- --debug
    Main->>Main: Detect --debug flag
    Main->>Config: load_config()
    Config-->>Main: Config

    Main->>Console: Print config settings
    Main->>Main: Create Tokio runtime

    loop Until Ctrl+C
        Main->>Collector: collect()
        Collector->>WMI: WMI Queries
        WMI-->>Collector: System data
        Collector-->>Main: CheckIn

        Main->>Console: Print JSON

        alt api_url is set
            Main->>Sender: send(checkin, url, tls_insecure)
            Sender->>Server: POST /checkin (JSON)
            Server-->>Sender: HTTP Response
            Sender-->>Main: Result
        end

        Main->>Main: Sleep interval_seconds

        alt Ctrl+C received
            Main->>Console: "Shutting down..."
            Main->>CLI: Exit
        end
    end
```

### Configuration Loading Flow

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Caller Module
    participant Config as config.rs
    participant FS as File System
    participant Env as Environment Variables

    Caller->>Config: load_config()
    Config->>Config: exe_dir()
    Config->>FS: Check config.toml exists

    alt config.toml exists
        Config->>FS: Read config.toml
        FS-->>Config: TOML content
        Config->>Config: Parse TOML to Config struct
    else config.toml missing
        Config->>FS: generate_template_config()
        FS-->>Config: Template written
        Config->>Config: Use Config::default()
    end

    Config->>Env: Check INVENTORY_API_URL
    alt Set
        Config->>Config: Override api_url
    end

    Config->>Env: Check INVENTORY_INTERVAL_SECONDS
    alt Set and valid u64
        Config->>Config: Override interval_seconds
    end

    Config->>Env: Check INVENTORY_TLS_INSECURE
    alt Set to "true" (case-insensitive)
        Config->>Config: Override tls_insecure = true
    end

    Config-->>Caller: Config
```

---

## Data Flow Summary

```mermaid
flowchart LR
    subgraph Collection["Data Collection"]
        ENV["COMPUTERNAME env"]
        NET["Network Interfaces"]
        WMI1["Win32_ComputerSystem"]
        WMI2["Win32_BIOS"]
        WMI3["Win32_DiskDrive"]
    end

    subgraph Processing["Processing"]
        COLLECT["collector::collect()"]
        CHECKIN["CheckIn struct"]
        JSON["JSON Serialization"]
    end

    subgraph Transmission["Transmission"]
        SEND["sender::send()"]
        HTTP["HTTP POST"]
        SERVER["Inventory Server"]
    end

    ENV --> COLLECT
    NET --> COLLECT
    WMI1 --> COLLECT
    WMI2 --> COLLECT
    WMI3 --> COLLECT

    COLLECT --> CHECKIN
    CHECKIN --> JSON
    JSON --> SEND
    SEND --> HTTP
    HTTP --> SERVER
```

---

## Configuration Hierarchy

```mermaid
flowchart TB
    subgraph Priority["Configuration Priority (highest to lowest)"]
        direction TB
        E["3. Environment Variables<br/>(INVENTORY_API_URL, etc.)"]
        F["2. config.toml file"]
        D["1. Code defaults<br/>(Config::default())"]
    end

    D --> F --> E

    style E fill:#90EE90
    style F fill:#87CEEB
    style D fill:#FFB6C1
```

---

## Error Handling Strategy

| Component | Error Behavior |
|-----------|----------------|
| `config::load_config()` | Returns `Result<Config>` with context |
| `collector::collect()` | Returns `Result<CheckIn>` with WMI error context |
| `sender::send()` | Returns `Result<()>` with HTTP status and body |
| `service::run_service()` | Silently continues on collector/sender errors |
| `main::run_test_mode()` | Propagates errors to caller |
| `main::run_debug_mode()` | Propagates errors to caller |

---

## External Dependencies

| Crate | Purpose |
|-------|---------|
| `windows-service` | Windows Service API bindings |
| `wmi` | Windows WMI query interface |
| `tokio` | Async runtime (multi-thread, signals) |
| `reqwest` | HTTP client with rustls-tls |
| `serde` / `serde_json` | JSON serialization |
| `toml` | TOML configuration parsing |
| `chrono` | Timestamp generation (RFC3339) |
| `get_if_addrs` | Network interface enumeration |
| `anyhow` | Error handling with context |

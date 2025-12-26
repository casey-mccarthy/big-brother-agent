//! Inventory Agent (Windows Service)
//!
//! Runs as a Windows Service named `InventoryAgent`. Periodically collects inventory data and POSTs
//! JSON to the configured API endpoint.

#[cfg(target_os = "windows")]
mod collector;
#[cfg(target_os = "windows")]
mod config;
#[cfg(target_os = "windows")]
mod models;
#[cfg(target_os = "windows")]
mod sender;
#[cfg(target_os = "windows")]
mod service;

use anyhow::Result;
#[cfg(target_os = "windows")]
use std::time::Duration;

fn main() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Error: This agent only runs on Windows.");
        std::process::exit(1);
    }

    #[cfg(target_os = "windows")]
    {
        // Check for --test flag for interactive development testing
        if std::env::args().any(|arg| arg == "--test") {
            return run_test_mode();
        }

        // Check for --debug flag for foreground mode with periodic check-ins
        if std::env::args().any(|arg| arg == "--debug") {
            return run_debug_mode();
        }

        // Service entry point.
        service::run()
    }
}

/// Debug mode: run in foreground with periodic check-ins and console output.
/// Run with: cargo run -- --debug
#[cfg(target_os = "windows")]
fn run_debug_mode() -> Result<()> {
    println!("[DEBUG] Starting inventory agent in debug mode...");

    let cfg = config::load_config()?;
    println!(
        "[DEBUG] Check-in interval: {} seconds",
        cfg.interval_seconds
    );

    if let Some(ref url) = cfg.api_url {
        println!("[DEBUG] API URL: {}", url);
    } else {
        println!("[DEBUG] WARNING: INVENTORY_API_URL not set - will collect but not send");
    }

    if cfg.tls_insecure {
        println!("[DEBUG] WARNING: TLS certificate validation is DISABLED (lab mode)");
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        loop {
            println!("\n[DEBUG] Collecting inventory...");
            match collector::collect() {
                Ok(checkin) => {
                    println!("[DEBUG] Collected data:");
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&checkin).unwrap_or_default()
                    );

                    if let Some(ref url) = cfg.api_url {
                        println!("\n[DEBUG] Sending check-in...");
                        match sender::send(&checkin, url, cfg.tls_insecure).await {
                            Ok(_) => println!("[DEBUG] Check-in sent successfully"),
                            Err(e) => println!("[DEBUG] Send failed: {}", e),
                        }
                    }
                }
                Err(e) => println!("[DEBUG] Collection failed: {}", e),
            }

            println!(
                "\n[DEBUG] Next check-in in {} seconds. Press Ctrl+C to exit.",
                cfg.interval_seconds
            );

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(cfg.interval_seconds)) => {}
                _ = tokio::signal::ctrl_c() => {
                    println!("\n[DEBUG] Shutting down...");
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Test mode: collect inventory and optionally send to server.
/// Run with: cargo run -- --test
#[cfg(target_os = "windows")]
fn run_test_mode() -> Result<()> {
    println!("=== Inventory Agent Test Mode ===\n");

    let cfg = config::load_config()?;

    println!("Collecting inventory...");
    let checkin = collector::collect()?;

    println!("\nCollected data:");
    println!("{}", serde_json::to_string_pretty(&checkin)?);

    // If API URL is set, attempt to send
    if let Some(ref url) = cfg.api_url {
        println!("\nSending to: {}", url);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(sender::send(&checkin, url, cfg.tls_insecure))?;
        println!("Send successful!");
    } else {
        println!("\nINVENTORY_API_URL not set - skipping send");
        println!(
            "Set it to test sending: $env:INVENTORY_API_URL=\"http://localhost:8443/checkin\""
        );
    }

    Ok(())
}

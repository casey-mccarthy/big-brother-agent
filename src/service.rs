use anyhow::{Context, Result};
use std::ffi::OsString;
use std::time::Duration;
use tokio::runtime::Runtime;
use windows_service::define_windows_service;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

use crate::{collector, config, sender};

const SERVICE_NAME: &str = "InventoryAgent";

define_windows_service!(ffi_service_main, service_main);

pub fn run() -> Result<()> {
    windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .context("service dispatcher start failed")?;
    Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        // Best-effort: in production, write to Event Log. Here we just swallow to avoid crash loops.
        let _ = e;
    }
}

fn run_service() -> Result<()> {
    // Load configuration (with env var overrides)
    let cfg = config::load_config()?;

    // Validate api_url is set
    let api_url = cfg.api_url.ok_or_else(|| {
        anyhow::anyhow!("INVENTORY_API_URL not set (required in config.toml or environment)")
    })?;

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
        .context("register service control handler failed")?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Run interval loop on a tokio runtime
    let rt = Runtime::new().context("tokio runtime create failed")?;

    rt.block_on(async move {
        loop {
            // Check for shutdown without blocking the async loop too aggressively
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            match collector::collect() {
                Ok(checkin) => {
                    if let Err(_e) = sender::send(&checkin, &api_url, cfg.tls_insecure).await {
                        // TODO: write to Windows Event Log
                    }
                }
                Err(_e) => {
                    // TODO: write to Windows Event Log
                }
            }

            tokio::time::sleep(Duration::from_secs(cfg.interval_seconds)).await;
        }
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

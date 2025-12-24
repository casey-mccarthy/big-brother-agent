use anyhow::{Context, Result};
use chrono::Utc;
use wmi::{COMLibrary, WMIConnection};

use crate::models::{CheckIn, Drive};

pub fn collect() -> Result<CheckIn> {
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN".to_string());

    let ip_address = primary_ipv4().unwrap_or_else(|| "0.0.0.0".to_string());

    let com = COMLibrary::new().context("Initialize COM library failed")?;
    let wmi = WMIConnection::new(com.into()).context("WMI connection failed")?;

    // Logged in user
    #[derive(serde::Deserialize, Debug)]
    struct Win32ComputerSystem {
        #[serde(rename = "UserName")]
        user_name: Option<String>,
    }
    let cs: Vec<Win32ComputerSystem> = wmi
        .raw_query("SELECT UserName FROM Win32_ComputerSystem")
        .context("WMI query Win32_ComputerSystem failed")?;
    let logged_in_user = cs.get(0).and_then(|x| x.user_name.clone());

    // Laptop serial (BIOS)
    #[derive(serde::Deserialize, Debug)]
    struct Win32Bios {
        #[serde(rename = "SerialNumber")]
        serial_number: Option<String>,
    }
    let bios: Vec<Win32Bios> = wmi
        .raw_query("SELECT SerialNumber FROM Win32_BIOS")
        .context("WMI query Win32_BIOS failed")?;
    let laptop_serial = bios
        .get(0)
        .and_then(|x| x.serial_number.clone())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    // Physical drives
    #[derive(serde::Deserialize, Debug)]
    struct Win32DiskDrive {
        #[serde(rename = "Model")]
        model: Option<String>,
        #[serde(rename = "SerialNumber")]
        serial_number: Option<String>,
        #[serde(rename = "DeviceID")]
        device_id: Option<String>,
    }

    let disks: Vec<Win32DiskDrive> = wmi
        .raw_query("SELECT Model, SerialNumber, DeviceID FROM Win32_DiskDrive")
        .context("WMI query Win32_DiskDrive failed")?;

    let drives = disks
        .into_iter()
        .map(|d| Drive {
            model: d.model.unwrap_or_else(|| "UNKNOWN".to_string()),
            serial_number: d.serial_number.map(|s| s.trim().to_string()),
            device_id: d.device_id.unwrap_or_else(|| "UNKNOWN".to_string()),
        })
        .collect::<Vec<_>>();

    Ok(CheckIn {
        hostname,
        ip_address,
        logged_in_user,
        laptop_serial,
        drives,
        timestamp_utc: Utc::now().to_rfc3339(),
    })
}

fn primary_ipv4() -> Option<String> {
    // Pragmatic approach for baseline. Replace with GetAdaptersAddresses if you need stronger fidelity.
    let ifaces = get_if_addrs::get_if_addrs().ok()?;
    for iface in ifaces {
        if iface.is_loopback() {
            continue;
        }
        if let std::net::IpAddr::V4(v4) = iface.ip() {
            let s = v4.to_string();
            if s.starts_with("169.254.") {
                continue;
            }
            return Some(s);
        }
    }
    None
}

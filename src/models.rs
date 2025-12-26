use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Drive {
    pub model: String,
    pub serial_number: Option<String>,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckIn {
    pub hostname: String,
    pub ip_address: String,
    pub logged_in_user: Option<String>,
    pub laptop_serial: String,
    pub drives: Vec<Drive>,
    pub timestamp_utc: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_clone() {
        let drive = Drive {
            model: "Test Drive".to_string(),
            serial_number: Some("SN123".to_string()),
            device_id: "DRIVE0".to_string(),
        };

        let cloned = drive.clone();
        assert_eq!(drive.model, cloned.model);
        assert_eq!(drive.serial_number, cloned.serial_number);
    }

    #[test]
    fn test_checkin_serialization() {
        let checkin = CheckIn {
            hostname: "TEST-HOST".to_string(),
            ip_address: "192.168.1.1".to_string(),
            logged_in_user: Some("testuser".to_string()),
            laptop_serial: "ABC123".to_string(),
            drives: vec![],
            timestamp_utc: "2025-12-18T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&checkin).unwrap();
        assert!(json.contains("TEST-HOST"));
        assert!(json.contains("ABC123"));
    }

    #[test]
    fn test_checkin_roundtrip() {
        let original = CheckIn {
            hostname: "ROUNDTRIP".to_string(),
            ip_address: "10.0.0.1".to_string(),
            logged_in_user: None,
            laptop_serial: "SERIAL".to_string(),
            drives: vec![Drive {
                model: "TestDrive".to_string(),
                serial_number: None,
                device_id: "DEVICE0".to_string(),
            }],
            timestamp_utc: "2025-12-18T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: CheckIn = serde_json::from_str(&json).unwrap();

        assert_eq!(original.hostname, parsed.hostname);
        assert_eq!(original.drives.len(), parsed.drives.len());
    }
}

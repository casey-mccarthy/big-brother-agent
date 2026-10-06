//! Wire-contract tests between inventory-agent and inventory-server.
//!
//! `tests/fixtures/checkin.json` is the canonical check-in payload. The same file
//! is committed to the inventory-server repository, where a test asserts the
//! server accepts exactly this JSON. Here we assert the agent produces exactly
//! this JSON. If either side changes the schema, one of the two suites goes red.

use inventory_agent::models::{CheckIn, Drive};
use inventory_agent::sender::send;
use mockito::{Matcher, Server};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/checkin.json");

fn fixture_value() -> Value {
    serde_json::from_str(FIXTURE).expect("fixture is valid JSON")
}

/// The CheckIn the collector would build for the machine described by the fixture.
fn fixture_checkin() -> CheckIn {
    CheckIn {
        hostname: "LAPTOP-ABC123".to_string(),
        ip_address: "192.168.1.100".to_string(),
        logged_in_user: Some("DOMAIN\\jsmith".to_string()),
        laptop_serial: "ABC123XYZ".to_string(),
        drives: vec![
            Drive {
                model: "Samsung SSD 970 EVO 500GB".to_string(),
                serial_number: Some("S4EVNX0M123456".to_string()),
                device_id: r"\\.\PHYSICALDRIVE0".to_string(),
            },
            Drive {
                model: "Generic USB Drive".to_string(),
                serial_number: None,
                device_id: r"\\.\PHYSICALDRIVE1".to_string(),
            },
        ],
        timestamp_utc: "2025-12-18T10:30:00Z".to_string(),
    }
}

#[test]
fn checkin_serializes_to_the_shared_contract() {
    // Exact equality: field names, nesting, and `null` for a missing drive
    // serial all have to match what the server's model expects.
    let actual = serde_json::to_value(fixture_checkin()).unwrap();
    assert_eq!(actual, fixture_value());
}

#[test]
fn timestamp_format_matches_the_contract() {
    // The server stores timestamps as text and sorts them lexicographically, so
    // the agent must emit a fixed-width RFC 3339 string: whole seconds, `Z` suffix.
    let ts = inventory_agent::models::utc_now_rfc3339();
    assert_eq!(ts.len(), "2025-12-18T10:30:00Z".len(), "got {ts}");
    assert!(ts.ends_with('Z'), "got {ts}");
    chrono::DateTime::parse_from_rfc3339(&ts).expect("parses as RFC 3339");
}

#[tokio::test]
async fn send_posts_the_contract_json_to_the_checkin_url() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/checkin")
        .match_header("content-type", "application/json")
        .match_body(Matcher::Json(fixture_value()))
        .with_status(200)
        .create_async()
        .await;

    let api_url = format!("{}/checkin", server.url());
    send(&fixture_checkin(), &api_url, false)
        .await
        .expect("send succeeds");

    mock.assert_async().await;
}

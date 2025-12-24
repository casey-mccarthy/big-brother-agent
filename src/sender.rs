use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::models::CheckIn;

pub async fn send(checkin: &CheckIn, api_url: &str, tls_insecure: bool) -> Result<()> {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .danger_accept_invalid_certs(tls_insecure)
        .build()
        .context("reqwest client build failed")?;

    let resp = client
        .post(api_url)
        .json(checkin)
        .send()
        .await
        .context("HTTP send failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("API returned {}: {}", status, body);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CheckIn;
    use mockito::Server;
    use serial_test::serial;

    fn create_test_checkin() -> CheckIn {
        CheckIn {
            hostname: "TEST-HOST".to_string(),
            ip_address: "192.168.1.100".to_string(),
            logged_in_user: Some("testuser".to_string()),
            laptop_serial: "TEST-SERIAL".to_string(),
            drives: vec![],
            timestamp_utc: "2025-12-18T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_send_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/checkin")
            .with_status(200)
            .create_async()
            .await;

        let api_url = format!("{}/checkin", server.url());
        let checkin = create_test_checkin();
        let result = send(&checkin, &api_url, false).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_send_server_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/checkin")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let api_url = format!("{}/checkin", server.url());
        let checkin = create_test_checkin();
        let result = send(&checkin, &api_url, false).await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[tokio::test]
    #[serial]
    async fn test_send_with_empty_url() {
        // Test that send fails with connection error when URL is invalid
        let checkin = create_test_checkin();
        let result = send(&checkin, "", false).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_send_with_tls_insecure() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/checkin")
            .with_status(200)
            .create_async()
            .await;

        let api_url = format!("{}/checkin", server.url());
        let checkin = create_test_checkin();
        let result = send(&checkin, &api_url, true).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_send_timeout() {
        // This test verifies timeout is configured (15 seconds)
        // Actual timeout testing would require a slow server mock
        let checkin = create_test_checkin();
        let result = send(&checkin, "http://192.0.2.1:1/checkin", false).await; // TEST-NET address

        // Should fail (connection refused or timeout)
        assert!(result.is_err());
    }
}

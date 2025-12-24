// Library exports for testing

pub mod config;
pub mod models;
pub mod sender;

// Note: collector and service modules require Windows-specific APIs and are not exported for cross-platform testing
#[cfg(target_os = "windows")]
pub mod collector;

#[cfg(target_os = "windows")]
pub mod service;

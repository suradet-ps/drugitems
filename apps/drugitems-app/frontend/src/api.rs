//! Frontend API layer - the only place the webview talks to the Tauri
//! backend.
//!
//! No hosts, no credentials, no SQL live here - every call is a thin
//! `invoke` to a Rust command, which owns all connection concerns.
//!
//! Errors cross the IPC as a typed [`ApiError`] (kind + Thai message):
//! components switch on `kind` (e.g. to raise the connection banner or the
//! "no snapshot" empty state) and display `message` verbatim.

use drugitems_core::CompareReport;
use serde::{Deserialize, Serialize};

use crate::state::ConnectionHealth;

/// Failure class of a backend command - mirrors the Rust
/// `CommandErrorKind` (camelCase over the wire).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiErrorKind {
    /// No connection settings stored.
    NotConfigured,
    /// MySQL could not be reached.
    Connection,
    /// The read-only guard rejected a statement - an internal error.
    Guard,
    /// The statement failed server-side.
    Query,
    /// The snapshot file could not be read/parsed.
    File,
}

/// A backend command failure: machine-readable kind + the Thai message to
/// show verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Failure class.
    pub kind: ApiErrorKind,
    /// User-facing message (Thai).
    pub message: String,
}

impl ApiError {
    fn from_bridge(err: drugitems_bridge::BridgeError) -> Self {
        if let drugitems_bridge::BridgeError::Command(text) = &err
            && let Ok(typed) = serde_json::from_str::<ApiError>(text)
        {
            return typed;
        }
        ApiError {
            kind: ApiErrorKind::Query,
            message: err.to_string(),
        }
    }
}

/// Plaintext connection settings typed by the operator. The Rust side
/// encrypts it before anything touches disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

/// Non-secret summary of the saved connection (password never returned).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
}

/// Non-secret app settings (mirrors the backend's `AppSettings`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub table_name: String,
    pub site_label: String,
    pub ignore_columns: Vec<String>,
    pub last_snapshot_file: Option<String>,
}

/// Editable subset of the app settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsInput {
    pub table_name: String,
    pub site_label: String,
    pub ignore_columns: Vec<String>,
}

/// Non-sensitive summary of the saved configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub configured: bool,
    pub site_label: Option<String>,
    pub host: Option<String>,
    pub database: Option<String>,
    pub user: Option<String>,
    pub table_name: String,
}

/// Result of the backend's `SELECT 1` smoke test.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
}

/// Non-secret description of the loaded snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub file_name: Option<String>,
    pub path: Option<String>,
    pub rows: usize,
    pub columns: Vec<String>,
}

async fn invoke_raw<T>(cmd: &str, args: impl Serialize) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    drugitems_bridge::invoke::<T>(cmd, args)
        .await
        .map_err(ApiError::from_bridge)
}

async fn call_empty<T>(cmd: &str) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    invoke_raw(cmd, serde_json::json!({})).await
}

async fn call_struct_arg<T>(cmd: &str, arg_name: &str, arg: &impl Serialize) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    invoke_raw(cmd, serde_json::json!({ arg_name: arg })).await
}

/// Whether stored settings exist.
pub async fn is_configured() -> Result<bool, ApiError> {
    call_empty("is_configured").await
}

/// Configuration status (label, host, table, …) - drives the top bar.
pub async fn get_app_status() -> Result<AppStatus, ApiError> {
    call_empty("get_app_status").await
}

/// Live connection health for the top-bar status dot.
pub async fn connection_health() -> Result<ConnectionHealth, ApiError> {
    call_empty("connection_health").await
}

/// Save the connection config (encrypted at rest) and connect.
pub async fn save_connection(config: &ConnectionInput) -> Result<(), ApiError> {
    call_struct_arg("save_connection", "config", config).await
}

/// The saved connection (without the password) - pre-fills the form.
pub async fn get_connection() -> Result<ConnectionInfo, ApiError> {
    call_empty("get_connection").await
}

/// Load the non-secret app settings.
pub async fn get_settings() -> Result<AppSettings, ApiError> {
    call_empty("get_settings").await
}

/// Save the non-secret app settings.
pub async fn save_settings(settings: &AppSettingsInput) -> Result<(), ApiError> {
    call_struct_arg("save_settings", "settings", settings).await
}

/// Test connectivity without saving.
pub async fn test_connection(
    config: Option<&ConnectionInput>,
) -> Result<ConnectionTestResult, ApiError> {
    let args = config.cloned();
    call_struct_arg("test_connection", "config", &args).await
}

/// Clear the saved configuration (forget connection + loaded snapshot).
pub async fn clear_site_config() -> Result<(), ApiError> {
    call_empty("clear_site_config").await
}

/// Open a native dialog and load the chosen snapshot; `None` when the
/// dialog is cancelled.
pub async fn choose_snapshot() -> Result<Option<SnapshotMeta>, ApiError> {
    call_empty("choose_snapshot").await
}

/// The currently loaded snapshot meta, if any.
pub async fn snapshot_info() -> Result<Option<SnapshotMeta>, ApiError> {
    call_empty("snapshot_info").await
}

/// Run the comparison against the loaded snapshot.
pub async fn run_compare() -> Result<CompareReport, ApiError> {
    call_empty("run_compare").await
}

/// Save the current report as CSV; returns the saved path.
pub async fn export_report() -> Result<String, ApiError> {
    call_empty("export_report").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bridge_parses_typed_command_error() {
        let typed = ApiError {
            kind: ApiErrorKind::File,
            message: "อ่านไฟล์ไม่สำเร็จ".into(),
        };
        let text = serde_json::to_string(&typed).unwrap();
        let err = ApiError::from_bridge(drugitems_bridge::BridgeError::Command(text));
        assert_eq!(err, typed);
    }

    #[test]
    fn from_bridge_falls_back_to_query_for_unparseable_payload() {
        let err = ApiError::from_bridge(drugitems_bridge::BridgeError::Command("not json".into()));
        assert_eq!(err.kind, ApiErrorKind::Query);
    }
}

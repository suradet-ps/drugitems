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
use zeroize::Zeroizing;

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
    /// Convert a bridge error into a typed `ApiError`, falling back to the
    /// `Query` kind when the backend payload is not a typed error.
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
    /// Wiped in place whenever this input is dropped or replaced.
    pub password: Zeroizing<String>,
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

/// Invoke a backend command with an already-serializable argument payload.
async fn invoke_raw<T>(cmd: &str, args: impl Serialize) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    drugitems_bridge::invoke::<T>(cmd, args)
        .await
        .map_err(ApiError::from_bridge)
}

/// Invoke with a pre-serialized, self-wiping JSON payload.
async fn call_serialized<T>(cmd: &str, payload: Zeroizing<String>) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    drugitems_bridge::invoke_serialized::<T>(cmd, payload)
        .await
        .map_err(ApiError::from_bridge)
}

/// Map a payload serialization failure to the same error shape the bridge
/// uses, so callers only ever see [`ApiError`].
fn serialize_error(err: serde_json::Error) -> ApiError {
    ApiError::from_bridge(drugitems_bridge::BridgeError::Serialize(err.to_string()))
}

/// Serialize a single named struct argument into one JSON object without
/// building an intermediate `serde_json::Value` (which would copy secrets).
///
/// The returned buffer wipes itself on drop; the caller should drop the
/// source value before awaiting the backend.
fn struct_arg_payload<T: Serialize + ?Sized>(
    arg_name: &str,
    arg: &T,
) -> Result<Zeroizing<String>, ApiError> {
    let mut buf = Zeroizing::new(Vec::<u8>::with_capacity(256));
    buf.push(b'{');
    serde_json::to_writer(&mut *buf, arg_name).map_err(serialize_error)?;
    buf.push(b':');
    serde_json::to_writer(&mut *buf, arg).map_err(serialize_error)?;
    buf.push(b'}');
    let bytes = std::mem::take(&mut *buf);
    let text = String::from_utf8(bytes).expect("invariant: serde_json emits UTF-8");
    Ok(Zeroizing::new(text))
}

/// Invoke a backend command that takes no arguments.
async fn call_empty<T>(cmd: &str) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    invoke_raw(cmd, serde_json::json!({})).await
}

/// Invoke a backend command with a single named struct argument.
async fn call_struct_arg<T>(cmd: &str, arg_name: &str, arg: &impl Serialize) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let payload = struct_arg_payload(arg_name, arg)?;
    call_serialized(cmd, payload).await
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
///
/// Takes ownership so the plaintext input (and its zeroizing password) is
/// dropped immediately after serialization - the in-flight call then only
/// holds the self-wiping payload.
pub async fn save_connection(config: ConnectionInput) -> Result<(), ApiError> {
    let payload = struct_arg_payload("config", &config)?;
    drop(config);
    call_serialized("save_connection", payload).await
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
///
/// Takes ownership so the plaintext input is dropped right after
/// serialization, as in [`save_connection`].
pub async fn test_connection(
    config: Option<ConnectionInput>,
) -> Result<ConnectionTestResult, ApiError> {
    let payload = struct_arg_payload("config", &config)?;
    drop(config);
    call_serialized("test_connection", payload).await
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

    /// A typed command error payload round-trips into the same `ApiError`.
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

    /// An unparseable command payload falls back to the `Query` kind.
    #[test]
    fn from_bridge_falls_back_to_query_for_unparseable_payload() {
        let err = ApiError::from_bridge(drugitems_bridge::BridgeError::Command("not json".into()));
        assert_eq!(err.kind, ApiErrorKind::Query);
    }

    fn sample_input() -> ConnectionInput {
        ConnectionInput {
            host: "10.0.0.1".to_string(),
            port: 3306,
            database: "hos".to_string(),
            user: "drug_ro".to_string(),
            password: Zeroizing::new("s3cret".to_string()),
        }
    }

    /// The secret-bearing payload is one wrapper object, written directly
    /// instead of through an intermediate `serde_json::Value`.
    #[test]
    fn struct_arg_payload_wraps_one_named_argument() {
        let payload =
            struct_arg_payload("config", &sample_input()).expect("serialize struct argument");
        let value: serde_json::Value =
            serde_json::from_str(&payload).expect("payload must be valid JSON");
        assert_eq!(value.as_object().map(|object| object.len()), Some(1));
        assert_eq!(value["config"]["user"], "drug_ro");
        assert_eq!(value["config"]["password"], "s3cret");
    }

    /// `None` must stay an explicit `null` argument, matching the backend's
    /// `Option<ConnectionInput>`.
    #[test]
    fn struct_arg_payload_serializes_none_as_null() {
        let payload =
            struct_arg_payload("config", &None::<ConnectionInput>).expect("serialize option");
        assert_eq!(&*payload, "{\"config\":null}");
    }
}

//! Tauri command adapters.
//!
//! Errors cross the IPC as a typed [`CommandError`] (kind + Thai message)
//! so the frontend can decide presentation from the kind - e.g. raise the
//! connection banner or the "no snapshot" empty state - instead of matching
//! on message text. Crates stay English and typed; this layer is the only
//! place where errors are translated for the UI.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use drugitems_config::{AppSettings, ConnectionConfig};
use drugitems_core::CompareReport;
use drugitems_db::{DbClient, DbConfig};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Ephemeral vault used when the OS keychain is unavailable (headless
/// environments). Config saved through it is never persisted to disk.
#[derive(Default)]
pub struct EphemeralVault {
    key: std::sync::Mutex<Option<encryptman::MasterKey>>,
}

impl drugitems_config::SecretVault for EphemeralVault {
    fn encrypt(&self, plaintext: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.key.lock().expect("invariant: ephemeral key lock");
        let key = guard.get_or_insert_with(|| {
            encryptman::MasterKey::generate().expect("invariant: OS rng available")
        });
        Ok(encryptman::encrypt(key, plaintext)?)
    }

    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let guard = self.key.lock().expect("invariant: ephemeral key lock");
        let key = guard
            .as_ref()
            .ok_or_else(|| Box::new(std::io::Error::other("no master key available")))?;
        Ok(encryptman::decrypt(key, ciphertext)?)
    }
}

/// Failure class of a command - the frontend switches on this for
/// presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandErrorKind {
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

/// User-facing command error: a machine-readable kind plus the Thai message
/// shown verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Failure class.
    pub kind: CommandErrorKind,
    /// User-facing message (Thai).
    pub message: String,
}

impl CommandError {
    fn new(kind: CommandErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Logs the underlying cause for developers and returns the user-facing
/// Thai error.
fn dev_log(
    context: &str,
    detail: &impl std::fmt::Debug,
    kind: CommandErrorKind,
    message: &'static str,
) -> CommandError {
    eprintln!("[drugitems] {context} failed: {detail:?}");
    CommandError::new(kind, message)
}

/// Maps a repository error to a typed command error. `action` is the Thai
/// verb phrase for the Query variant.
fn map_db_error(err: drugitems_db::Error, action: &'static str) -> CommandError {
    eprintln!("[drugitems] {action} failed: {err:?}");
    match err {
        drugitems_db::Error::Connect { .. } => {
            CommandError::new(CommandErrorKind::Connection, "เชื่อมต่อฐานข้อมูล MySQL ไม่สำเร็จ")
        }
        drugitems_db::Error::Database(sqlx::Error::PoolTimedOut)
        | drugitems_db::Error::Database(sqlx::Error::PoolClosed)
        | drugitems_db::Error::Database(sqlx::Error::Io(_)) => {
            CommandError::new(CommandErrorKind::Connection, "เชื่อมต่อฐานข้อมูล MySQL ไม่สำเร็จ")
        }
        drugitems_db::Error::Database(sqlx::Error::Database(db)) => {
            CommandError::new(CommandErrorKind::Query, format!("{action}ไม่สำเร็จ ({db})"))
        }
        drugitems_db::Error::ReadOnlyViolation(_) => {
            CommandError::new(CommandErrorKind::Guard, "ระบบความปลอดภัยของแอปปฏิเสธคำสั่งนี้")
        }
        drugitems_db::Error::NotFound(msg) => {
            CommandError::new(CommandErrorKind::Query, format!("{action}ไม่สำเร็จ ({msg})"))
        }
        other => CommandError::new(
            CommandErrorKind::Query,
            format!("{action}ไม่สำเร็จ ({other})"),
        ),
    }
}

/// Maps a snapshot load error to a user-facing file error.
fn map_snapshot_error(err: drugitems_snapshot::SnapshotError) -> CommandError {
    CommandError::new(
        CommandErrorKind::File,
        format!("อ่านไฟล์ snapshot ไม่สำเร็จ - {err}"),
    )
}

/// Connection settings submitted from the settings dialog. The password
/// lives in this struct only for the duration of the command call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    /// Hostname or IP.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Database user.
    pub user: String,
    /// Database password.
    pub password: String,
}

impl From<ConnectionInput> for ConnectionConfig {
    fn from(i: ConnectionInput) -> Self {
        Self {
            host: i.host,
            port: i.port,
            database: i.database,
            user: i.user,
            password: SecretString::from(i.password),
        }
    }
}

/// App settings submitted from the settings dialog (non-secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsInput {
    /// Table to compare against the snapshot.
    pub table_name: String,
    /// Site label shown in the top bar.
    pub site_label: String,
    /// Columns excluded from the cell-by-cell comparison.
    pub ignore_columns: Vec<String>,
}

impl From<AppSettingsInput> for AppSettings {
    fn from(i: AppSettingsInput) -> Self {
        Self {
            table_name: i.table_name,
            site_label: i.site_label,
            ignore_columns: i.ignore_columns,
            last_snapshot_file: None,
        }
    }
}

/// Non-secret summary of the saved connection (password never returned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    /// Hostname or IP.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Database user.
    pub user: String,
}

/// Non-sensitive summary of the saved configuration (never includes the
/// password).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    /// Whether a config has been saved.
    pub configured: bool,
    /// Site label, if configured.
    pub site_label: Option<String>,
    /// Host, if configured.
    pub host: Option<String>,
    /// Database name, if configured.
    pub database: Option<String>,
    /// Database user, if configured.
    pub user: Option<String>,
    /// Table being compared.
    pub table_name: String,
}

/// Live MySQL reachability, polled by the frontend - drives the top-bar
/// status dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    /// No stored settings - the settings dialog is the flow.
    Unconfigured,
    /// A ping succeeded recently.
    Connected,
    /// MySQL could not be reached.
    Disconnected,
}

/// Result of the backend's `SELECT 1` smoke test.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    /// Round-trip latency of the ping.
    pub latency_ms: u64,
}

/// Non-secret description of the loaded snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    /// File name shown in the UI.
    pub file_name: Option<String>,
    /// Full path of the chosen file (only right after picking).
    pub path: Option<String>,
    /// Number of data rows.
    pub rows: usize,
    /// Column names in file order.
    pub columns: Vec<String>,
}

/// Overall timeout for a connect+ping round trip.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

/// Connects a client with a wall-clock timeout, mapping timeouts to a
/// user-facing connection error instead of hanging the command.
async fn connect_client(cfg: DbConfig, action: &'static str) -> Result<DbClient, CommandError> {
    match tokio::time::timeout(COMMAND_TIMEOUT, DbClient::connect(cfg)).await {
        Ok(Ok(client)) => Ok(client),
        Ok(Err(e)) => Err(map_db_error(e, action)),
        Err(_) => Err(CommandError::new(
            CommandErrorKind::Connection,
            format!("{action}หมดเวลา - ตรวจสอบ Host/Port และเครือข่าย"),
        )),
    }
}

/// Build the connection config for the MySQL client from the stored
/// connection config.
fn to_db_config(conn: &ConnectionConfig) -> DbConfig {
    DbConfig {
        host: conn.host.clone(),
        port: conn.port,
        database: conn.database.clone(),
        user: conn.user.clone(),
        password: conn.password.clone(),
    }
}

/// Load the stored connection config, mapping a missing file to the
/// user-facing NotConfigured error.
fn stored_connection(state: &AppState) -> Result<ConnectionConfig, CommandError> {
    state.store.load_connection().map_err(|_| {
        CommandError::new(CommandErrorKind::NotConfigured, "ยังไม่ได้ตั้งค่าการเชื่อมต่อ MySQL")
    })
}

/// Resolve a usable MySQL client, connecting from the saved config if
/// needed. `action` labels the user-facing error.
///
/// The cache lock is only held for the short cache lookup/store - the slow
/// work (keychain decrypt, TCP connect) happens with no lock held.
async fn client(state: &AppState, action: &'static str) -> Result<DbClient, CommandError> {
    {
        let guard = state.client.read().await;
        if let Some(client) = guard.as_ref() {
            return Ok(client.clone());
        }
    }

    let conn = stored_connection(state)?;
    let client = connect_client(to_db_config(&conn), action).await?;

    let mut slot = state.client.write().await;
    if slot.is_none() {
        *slot = Some(client.clone());
    }
    Ok(client)
}

/// Report the current configuration status (password never included).
#[tauri::command]
pub async fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, CommandError> {
    let settings = state.store.load_settings().map_err(|e| {
        dev_log(
            "get_app_status",
            &e,
            CommandErrorKind::Query,
            "อ่านการตั้งค่าไม่สำเร็จ",
        )
    })?;
    match state.store.load_connection() {
        Ok(conn) => Ok(AppStatus {
            configured: true,
            site_label: Some(settings.site_label),
            host: Some(conn.host),
            database: Some(conn.database),
            user: Some(conn.user),
            table_name: settings.table_name,
        }),
        Err(drugitems_config::Error::NoConfig) => Ok(AppStatus {
            configured: false,
            site_label: Some(settings.site_label),
            host: None,
            database: None,
            user: None,
            table_name: settings.table_name,
        }),
        Err(e) => Err(dev_log(
            "get_app_status",
            &e,
            CommandErrorKind::Query,
            "อ่านการตั้งค่าไม่สำเร็จ",
        )),
    }
}

/// Whether connection settings exist.
#[tauri::command]
pub async fn is_configured(state: State<'_, AppState>) -> Result<bool, CommandError> {
    Ok(state.store.connection_exists())
}

/// Live connection health for the top-bar status dot.
#[tauri::command]
pub async fn connection_health(
    state: State<'_, AppState>,
) -> Result<ConnectionHealth, CommandError> {
    if !state.store.connection_exists() {
        return Ok(ConnectionHealth::Unconfigured);
    }
    match client(&state, "ตรวจสอบการเชื่อมต่อ").await {
        Ok(c) => match c.ping().await {
            Ok(()) => {
                *state.health.write().await = ConnectionHealth::Connected;
                Ok(ConnectionHealth::Connected)
            }
            Err(_) => {
                *state.health.write().await = ConnectionHealth::Disconnected;
                Ok(ConnectionHealth::Disconnected)
            }
        },
        Err(e) => match e.kind {
            CommandErrorKind::NotConfigured => Ok(ConnectionHealth::Unconfigured),
            _ => Ok(ConnectionHealth::Disconnected),
        },
    }
}

/// Save the MySQL connection config (encrypted at rest) and connect.
#[tauri::command]
pub async fn save_connection(
    state: State<'_, AppState>,
    config: ConnectionInput,
) -> Result<(), CommandError> {
    let connection: ConnectionConfig = config.into();

    // Connect before persisting so a bad password never gets saved.
    let client = connect_client(to_db_config(&connection), "บันทึกการตั้งค่า").await?;

    // Encrypting for disk touches the OS keychain, which can stall on a
    // permission dialog with unsigned dev binaries - cap it as well.
    let saved = tokio::time::timeout(COMMAND_TIMEOUT, async {
        state.store.save_connection(&connection)
    })
    .await;
    match saved {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return Err(dev_log(
                "save_connection",
                &e,
                CommandErrorKind::Query,
                "บันทึกการตั้งค่าไม่สำเร็จ",
            ));
        }
        Err(_) => {
            return Err(CommandError::new(
                CommandErrorKind::Query,
                "บันทึกการตั้งค่าไม่สำเร็จ - การเข้าถึง Keychain ใช้เวลานานเกินไป",
            ));
        }
    }

    // Replace the cached client under a short lock; the previous pool is
    // closed after the lock is released.
    let old = {
        let mut slot = state.client.write().await;
        slot.replace(client.clone())
    };
    if let Some(old) = old {
        old.disconnect().await;
    }
    tracing::info!(host = %connection.host, "connection saved and connected");
    Ok(())
}

/// The saved connection, without the password - pre-fills the settings form.
#[tauri::command]
pub async fn get_connection(state: State<'_, AppState>) -> Result<ConnectionInfo, CommandError> {
    let conn = stored_connection(&state)?;
    Ok(ConnectionInfo {
        host: conn.host,
        port: conn.port,
        database: conn.database,
        user: conn.user,
    })
}

/// The non-secret app settings.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    state.store.load_settings().map_err(|e| {
        dev_log(
            "get_settings",
            &e,
            CommandErrorKind::Query,
            "อ่านการตั้งค่าไม่สำเร็จ",
        )
    })
}

/// Save the non-secret app settings as plain JSON, preserving the
/// last-loaded snapshot file name.
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettingsInput,
) -> Result<(), CommandError> {
    let mut settings: AppSettings = settings.into();
    if let Ok(existing) = state.store.load_settings() {
        settings.last_snapshot_file = existing.last_snapshot_file;
    }
    state.store.save_settings(&settings).map_err(|e| {
        dev_log(
            "save_settings",
            &e,
            CommandErrorKind::Query,
            "บันทึกการตั้งค่าไม่สำเร็จ",
        )
    })
}

/// Validate connectivity against the given settings (without saving) or the
/// saved configuration when no settings are provided.
#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    config: Option<ConnectionInput>,
) -> Result<ConnectionTestResult, CommandError> {
    let conn = match config {
        Some(input) => ConnectionConfig::from(input),
        None => stored_connection(&state)?,
    };

    let started = Instant::now();
    let client = connect_client(to_db_config(&conn), "ทดสอบการเชื่อมต่อ").await?;
    let ping = tokio::time::timeout(COMMAND_TIMEOUT, client.ping()).await;
    match ping {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(map_db_error(e, "ทดสอบการเชื่อมต่อ")),
        Err(_) => {
            return Err(CommandError::new(
                CommandErrorKind::Connection,
                "ทดสอบการเชื่อมต่อหมดเวลา - ตรวจสอบ Host/Port และเครือข่าย",
            ));
        }
    }
    client.disconnect().await;

    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

/// Remove the saved configuration and drop the connection pool.
#[tauri::command]
pub async fn clear_site_config(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.client.write().await = None;
    *state.snapshot.write().await = None;
    *state.report.write().await = None;
    state.store.clear().map_err(|e| {
        dev_log(
            "clear_site_config",
            &e,
            CommandErrorKind::Query,
            "ลบการตั้งค่าไม่สำเร็จ",
        )
    })
}

/// Open a native file dialog, load the chosen `.xls`/`.xlsx` snapshot into
/// memory, and remember it. Returns `None` when the dialog is cancelled.
#[tauri::command]
pub async fn choose_snapshot(
    state: State<'_, AppState>,
) -> Result<Option<SnapshotMeta>, CommandError> {
    let file = match rfd::AsyncFileDialog::new()
        .set_title("เลือกไฟล์ snapshot - ข้อมูลที่เชื่อถือได้ (source of truth)")
        .add_filter("Excel", &["xls", "xlsx"])
        .add_filter("ทั้งหมด", &["*"])
        .pick_file()
        .await
    {
        Some(handle) => handle.path().to_path_buf(),
        None => return Ok(None),
    };

    let path = file.clone();
    let loaded =
        tauri::async_runtime::spawn_blocking(move || drugitems_snapshot::load_snapshot(&path))
            .await
            .map_err(|e| {
                dev_log(
                    "choose_snapshot",
                    &e,
                    CommandErrorKind::File,
                    "อ่านไฟล์ snapshot ไม่สำเร็จ",
                )
            })?;
    let snapshot = loaded.map_err(map_snapshot_error)?;

    let meta = snapshot_meta_of(&snapshot, Some(file.display().to_string()));
    *state.snapshot.write().await = Some(snapshot);

    if let Ok(mut settings) = state.store.load_settings() {
        settings.last_snapshot_file = meta.file_name.clone();
        let _ = state.store.save_settings(&settings);
    }

    tracing::info!(
        rows = meta.rows,
        columns = meta.columns.len(),
        "snapshot loaded"
    );
    Ok(Some(meta))
}

/// The loaded snapshot's meta, if any.
#[tauri::command]
pub async fn snapshot_info(
    state: State<'_, AppState>,
) -> Result<Option<SnapshotMeta>, CommandError> {
    let guard = state.snapshot.read().await;
    Ok(guard.as_ref().map(|s| snapshot_meta_of(s, None)))
}

fn snapshot_meta_of(
    snapshot: &drugitems_core::SnapshotTable,
    path: Option<String>,
) -> SnapshotMeta {
    SnapshotMeta {
        file_name: snapshot.file_name.clone(),
        path,
        rows: snapshot.rows.len(),
        columns: snapshot.columns.clone(),
    }
}

/// Run the comparison: load the configured table from MySQL, compare it
/// cell-by-cell against the loaded snapshot, and cache the report.
#[tauri::command]
pub async fn run_compare(state: State<'_, AppState>) -> Result<CompareReport, CommandError> {
    let settings = state.store.load_settings().map_err(|e| {
        dev_log(
            "run_compare",
            &e,
            CommandErrorKind::Query,
            "อ่านการตั้งค่าไม่สำเร็จ",
        )
    })?;

    let snapshot = state.snapshot.read().await.clone().ok_or_else(|| {
        CommandError::new(
            CommandErrorKind::File,
            "ยังไม่ได้เลือกไฟล์ snapshot - เลือกไฟล์ก่อนเปรียบเทียบ",
        )
    })?;

    let client = client(&state, "เปรียบเทียบข้อมูล").await?;
    let db_table = client
        .load_table(&settings.table_name)
        .await
        .map_err(|e| map_db_error(e, "อ่านข้อมูลจากฐานข้อมูล"))?;

    let ignore: HashSet<String> = settings.ignore_columns.iter().cloned().collect();
    let mut report = drugitems_core::compare(&db_table, &snapshot, &ignore);

    let database = state
        .store
        .load_connection()
        .map(|c| c.database)
        .unwrap_or_default();
    report.db_table = Some(format!("{}.{}", database, settings.table_name));

    *state.report.write().await = Some(report.clone());
    tracing::info!(
        db_rows = report.db_rows,
        snapshot_rows = report.snapshot_rows,
        changed = report.counts.changed,
        added = report.counts.added_in_db,
        missing = report.counts.missing_in_db,
        "comparison done"
    );
    Ok(report)
}

/// Save the current comparison report as CSV through a native dialog.
/// Returns the saved path.
#[tauri::command]
pub async fn export_report(state: State<'_, AppState>) -> Result<String, CommandError> {
    let report =
        state.report.read().await.clone().ok_or_else(|| {
            CommandError::new(CommandErrorKind::File, "ยังไม่มีผลการเปรียบเทียบให้บันทึก")
        })?;
    let csv = crate::report::build_csv(&report);
    let path = rfd::AsyncFileDialog::new()
        .set_title("บันทึกรายงานความแตกต่าง")
        .set_file_name("drugitems-diff.csv")
        .add_filter("CSV", &["csv"])
        .save_file()
        .await
        .map(|handle| handle.path().to_path_buf())
        .ok_or_else(|| CommandError::new(CommandErrorKind::File, "ยกเลิกการบันทึกรายงาน"))?;
    std::fs::write(&path, csv).map_err(|e| {
        dev_log(
            "export_report",
            &e,
            CommandErrorKind::Query,
            "เขียนไฟล์รายงานไม่สำเร็จ",
        )
    })?;
    tracing::info!("report exported");
    Ok(path.display().to_string())
}

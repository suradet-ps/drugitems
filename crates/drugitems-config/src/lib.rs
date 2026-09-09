//! DrugItems configuration store - two independent JSON files under the
//! platform config directory:
//!
//! * `connection.json` - MySQL host/port/database/user/password, stored
//!   **encrypted** (AES-256-GCM + HKDF via `encryptman`, master key in the
//!   OS keychain via `encryptman-keyring`). Credentials never touch disk in
//!   plaintext.
//! * `settings.json` - non-secret app settings: which table to compare,
//!   a site label, the operator's ignored-column list, and the last-loaded
//!   snapshot file name. Plain readable JSON.

use std::fs;
use std::path::{Path, PathBuf};

use encryptman_keyring::Vault;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Keyring service name - shared by all DrugItems users on this machine.
const KEYRING_SERVICE: &str = "DrugItems";

/// Encrypted connection settings file name.
pub const CONNECTION_FILE: &str = "connection.json";
/// Plain app settings file name.
pub const SETTINGS_FILE: &str = "settings.json";

/// MySQL connection settings - the encrypted half of the store.
///
/// `PartialEq` is intentionally not derived: the password is a
/// [`SecretString`] and must not be compared/logged incidentally.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Hostname or IP.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Database user.
    pub user: String,
    /// Database password (never serialized in plaintext).
    pub password: SecretString,
}

/// Plaintext payload - the shape written inside the encrypted blob.
#[derive(Debug, Serialize, Deserialize)]
struct ConnectionConfigRaw {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

/// Non-secret app settings - the plain JSON half of the store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    /// Name of the table compared against the snapshot (default `drugitems`).
    pub table_name: String,
    /// Human-readable site label shown in the top bar (optional).
    pub site_label: String,
    /// Columns excluded from the cell-by-cell comparison. Empty = compare
    /// every column (the default stance - the snapshot is the truth).
    pub ignore_columns: Vec<String>,
    /// File name of the last loaded snapshot, for convenience/relaunch.
    pub last_snapshot_file: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            table_name: "drugitems".into(),
            site_label: String::new(),
            ignore_columns: Vec::new(),
            last_snapshot_file: None,
        }
    }
}

/// File wrapper around the encrypted blob.
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedFile {
    version: u32,
    ciphertext: String,
}

/// Errors produced by the config store.
#[derive(Debug, Error)]
pub enum Error {
    /// The OS keychain is unavailable or rejected the operation.
    #[error("keychain error: {0}")]
    Keychain(#[from] encryptman_keyring::Error),

    /// Encryption/decryption failed.
    #[error("crypto error: {0}")]
    Crypto(#[from] encryptman::CryptoError),

    /// The config file could not be read/written.
    #[error("config file error at {path}: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The stored payload is not valid JSON.
    #[error("config file is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// The vault (keychain) rejected the operation.
    #[error("vault error: {0}")]
    Vault(String),

    /// The stored payload uses an unsupported format version.
    #[error("unsupported config version: {0}")]
    UnsupportedVersion(u32),

    /// No connection config file exists.
    #[error("no connection configuration saved")]
    NoConfig,
}

/// Result alias for the config store.
pub type Result<T> = std::result::Result<T, Error>;

/// Abstraction over the master-key vault so the store is testable without
/// touching the OS keychain.
pub trait SecretVault: Send + Sync {
    /// Encrypt a plaintext string.
    fn encrypt(
        &self,
        plaintext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    /// Decrypt a ciphertext string.
    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// Default vault: OS keychain-backed master key.
pub struct KeyringVault {
    vault: Vault,
}

impl KeyringVault {
    /// Open (or create on first use) the OS keychain vault.
    pub fn new() -> Result<Self> {
        Ok(Self {
            vault: Vault::new(KEYRING_SERVICE)?,
        })
    }
}

impl SecretVault for KeyringVault {
    fn encrypt(
        &self,
        plaintext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.vault.encrypt(plaintext).map_err(Box::from)
    }

    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.vault.decrypt(ciphertext).map_err(Box::from)
    }
}

/// Configuration store bound to a directory, managing two JSON files:
/// [`CONNECTION_FILE`] (encrypted) and [`SETTINGS_FILE`] (plain).
pub struct ConfigStore {
    vault: Box<dyn SecretVault>,
    dir: PathBuf,
}

impl ConfigStore {
    /// Default store: keyring vault + platform config directory.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Keychain`] when the OS keychain is unavailable.
    pub fn open() -> Result<Self> {
        let vault = KeyringVault::new()?;
        Ok(Self::with_vault(Box::new(vault), default_config_dir()))
    }

    /// Store with an explicit vault and directory (used by tests and
    /// headless environments).
    pub fn with_vault(vault: Box<dyn SecretVault>, dir: PathBuf) -> Self {
        Self { vault, dir }
    }

    /// Absolute path of the encrypted connection file.
    pub fn connection_path(&self) -> PathBuf {
        self.dir.join(CONNECTION_FILE)
    }

    /// Absolute path of the plain settings file.
    pub fn settings_path(&self) -> PathBuf {
        self.dir.join(SETTINGS_FILE)
    }

    /// Whether a connection config has been saved.
    pub fn connection_exists(&self) -> bool {
        self.connection_path().exists()
    }

    /// Load and decrypt the connection config.
    ///
    /// Returns [`Error::NoConfig`] when none has been saved yet.
    pub fn load_connection(&self) -> Result<ConnectionConfig> {
        let raw = read_optional(&self.connection_path())?.ok_or(Error::NoConfig)?;
        let file: EncryptedFile = serde_json::from_str(&raw)?;
        if file.version != 1 {
            return Err(Error::UnsupportedVersion(file.version));
        }
        let plaintext = self
            .vault
            .decrypt(&file.ciphertext)
            .map_err(|e| Error::Vault(e.to_string()))?;
        let raw_config: ConnectionConfigRaw = serde_json::from_str(&plaintext)?;
        Ok(raw_config.into())
    }

    /// Encrypt and persist the connection config.
    pub fn save_connection(&self, config: &ConnectionConfig) -> Result<()> {
        let raw_config: ConnectionConfigRaw = config.clone().into();
        let plaintext = serde_json::to_string(&raw_config)?;
        let ciphertext = self
            .vault
            .encrypt(&plaintext)
            .map_err(|e| Error::Vault(e.to_string()))?;
        let file = EncryptedFile {
            version: 1,
            ciphertext,
        };
        write_json(&self.connection_path(), &file)
    }

    /// Load the app settings; returns [`AppSettings::default`] when no
    /// settings file exists yet.
    pub fn load_settings(&self) -> Result<AppSettings> {
        let path = self.settings_path();
        let raw = match read_optional(&path)? {
            Some(raw) => raw,
            None => return Ok(AppSettings::default()),
        };
        if raw.trim().is_empty() {
            return Ok(AppSettings::default());
        }
        Ok(serde_json::from_str(&raw)?)
    }

    /// Persist the app settings as plain JSON.
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        write_json(&self.settings_path(), settings)
    }

    /// Remove both config files.
    pub fn clear(&self) -> Result<()> {
        remove_optional(&self.connection_path())?;
        remove_optional(&self.settings_path())?;
        Ok(())
    }
}

impl From<ConnectionConfig> for ConnectionConfigRaw {
    fn from(c: ConnectionConfig) -> Self {
        Self {
            host: c.host,
            port: c.port,
            database: c.database,
            user: c.user,
            password: c.password.expose_secret().to_string(),
        }
    }
}

impl From<ConnectionConfigRaw> for ConnectionConfig {
    fn from(r: ConnectionConfigRaw) -> Self {
        Self {
            host: r.host,
            port: r.port,
            database: r.database,
            user: r.user,
            password: SecretString::from(r.password),
        }
    }
}

/// Platform config directory for the app (`~/Library/Application Support/DrugItems`
/// on macOS, `~/.config/DrugItems` on Linux, `%APPDATA%\DrugItems` on Windows).
fn default_config_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "DrugItems")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn map_io(source: std::io::Error, path: &Path) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Read a file, mapping a missing file to `Ok(None)`.
fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(map_io(e, path)),
    }
}

/// Remove a file, ignoring a missing file.
fn remove_optional(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(map_io(e, path)),
    }
}

/// Serialize `value` as pretty JSON and write it (creating the parent
/// directory).
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| map_io(e, path))?;
    }
    fs::write(path, json).map_err(|e| map_io(e, path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Test vault keeping a master key in memory.
    struct InMemoryVault {
        key: encryptman::MasterKey,
    }

    impl InMemoryVault {
        fn new() -> Self {
            Self {
                key: encryptman::MasterKey::generate().unwrap(),
            }
        }
    }

    impl SecretVault for InMemoryVault {
        fn encrypt(
            &self,
            plaintext: &str,
        ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(encryptman::encrypt(&self.key, plaintext)?)
        }

        fn decrypt(
            &self,
            ciphertext: &str,
        ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(encryptman::decrypt(&self.key, ciphertext)?)
        }
    }

    static DIR_LOCK: Mutex<()> = Mutex::new(());

    fn sample_connection() -> ConnectionConfig {
        ConnectionConfig {
            host: "10.0.0.5".into(),
            port: 3306,
            database: "hos".into(),
            user: "drug_ro".into(),
            password: SecretString::from("sup3r-s3cret"),
        }
    }

    fn store_in(dir: &tempfile::TempDir) -> ConfigStore {
        ConfigStore::with_vault(Box::new(InMemoryVault::new()), dir.path().to_path_buf())
    }

    #[test]
    fn connection_roundtrip_no_plaintext_on_disk() {
        let _guard = DIR_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = store_in(&dir);

        assert!(matches!(store.load_connection(), Err(Error::NoConfig)));
        store.save_connection(&sample_connection()).unwrap();
        assert!(store.connection_exists());

        let loaded = store.load_connection().unwrap();
        assert_eq!(loaded.host, "10.0.0.5");
        assert_eq!(loaded.port, 3306);
        assert_eq!(loaded.password.expose_secret(), "sup3r-s3cret");

        let on_disk = fs::read_to_string(store.connection_path()).unwrap();
        assert!(!on_disk.contains("sup3r-s3cret"), "plaintext password!");
        assert!(!on_disk.contains("10.0.0.5"), "plaintext host!");
    }

    #[test]
    fn settings_roundtrip_and_defaults() {
        let _guard = DIR_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = store_in(&dir);

        assert_eq!(store.load_settings().unwrap(), AppSettings::default());
        let settings = AppSettings {
            table_name: "drugitems".into(),
            site_label: "รพ.ทดสอบ".into(),
            ignore_columns: vec!["last_update".into()],
            last_snapshot_file: Some("drugiterms-20260908.xls".into()),
        };
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
        let on_disk = fs::read_to_string(store.settings_path()).unwrap();
        assert!(on_disk.contains("รพ.ทดสอบ"), "settings are plain by design");
    }

    #[test]
    fn wrong_vault_cannot_decrypt() {
        let _guard = DIR_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = store_in(&dir);
        store.save_connection(&sample_connection()).unwrap();
        let other = ConfigStore::with_vault(
            Box::new(InMemoryVault::new()),
            dir.path().to_path_buf(),
        );
        assert!(other.load_connection().is_err());
    }

    #[test]
    fn clear_removes_both_files() {
        let _guard = DIR_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = store_in(&dir);
        store.save_connection(&sample_connection()).unwrap();
        store.save_settings(&AppSettings::default()).unwrap();
        store.clear().unwrap();
        assert!(matches!(store.load_connection(), Err(Error::NoConfig)));
        assert!(!store.settings_path().exists());
    }
}

//! Shared application state for the Tauri backend.

use drugitems_config::ConfigStore;
use drugitems_core::{CompareReport, SnapshotTable};
use drugitems_db::DbClient;
use tokio::sync::RwLock;

/// Backend state managed by Tauri.
pub struct AppState {
    /// Encrypted config store (keyring-backed).
    pub store: ConfigStore,
    /// Lazily-created MySQL client for the saved config.
    pub client: RwLock<Option<DbClient>>,
    /// Cached connection health, refreshed by `connection_health`.
    pub health: RwLock<crate::commands::ConnectionHealth>,
    /// The loaded snapshot (source of truth), in memory.
    pub snapshot: RwLock<Option<SnapshotTable>>,
    /// The last comparison result, in memory.
    pub report: RwLock<Option<CompareReport>>,
}

impl AppState {
    /// Open the default config store.
    ///
    /// Falls back to an in-memory store if the OS keychain is unavailable
    /// (headless environments) so the app still boots.
    pub fn new() -> Self {
        let store = match ConfigStore::open() {
            Ok(store) => store,
            Err(e) => {
                tracing::warn!("keychain unavailable, falling back to in-memory store: {e}");
                ConfigStore::with_vault(
                    Box::new(crate::commands::EphemeralVault::default()),
                    std::path::PathBuf::new(),
                )
            }
        };
        Self {
            store,
            client: RwLock::new(None),
            health: RwLock::new(crate::commands::ConnectionHealth::Unconfigured),
            snapshot: RwLock::new(None),
            report: RwLock::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

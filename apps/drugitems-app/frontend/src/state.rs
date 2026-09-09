//! Client-side application state (signals shared between components).

use leptos::prelude::*;

use crate::api::SnapshotMeta;

/// Live MySQL reachability, mirrored from the backend's `connection_health`
/// command - drives the top-bar status dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    /// No stored settings - the setup screen is the flow.
    Unconfigured,
    /// A ping succeeded recently.
    Connected,
    /// MySQL could not be reached.
    Disconnected,
}

/// Shared state for the single-page flow. `Copy` because all fields are
/// copyable signals.
#[derive(Debug, Clone, Copy)]
pub struct AppState {
    /// Whether encrypted MySQL connection settings exist on this machine.
    pub configured: RwSignal<bool>,
    /// Whether the settings dialog is open.
    pub settings_open: RwSignal<bool>,
    /// Whether the user manual (วิธีใช้) dialog is open.
    pub help_open: RwSignal<bool>,
    /// Polled live reachability - top-bar dot source.
    pub health: RwSignal<ConnectionHealth>,
    /// Site label, mirrored from settings.
    pub site_label: RwSignal<String>,
    /// Table being compared, mirrored from settings.
    pub table_name: RwSignal<String>,
    /// Columns excluded from comparison.
    pub ignore_columns: RwSignal<Vec<String>>,
    /// The loaded snapshot meta (source of truth).
    pub snapshot: RwSignal<Option<SnapshotMeta>>,
    /// The last comparison report, if any.
    pub report: RwSignal<Option<drugitems_core::CompareReport>>,
    /// Whether a comparison is in flight.
    pub comparing: RwSignal<bool>,
    /// Last comparison error message, if any.
    pub compare_error: RwSignal<Option<String>>,
    /// A transient success banner ("เปรียบเทียบเสร็จสิ้น", "บันทึกแล้ว").
    pub last_action: RwSignal<Option<String>>,
}

impl AppState {
    /// Fresh state for a new app session.
    pub fn new() -> Self {
        Self {
            configured: RwSignal::new(false),
            settings_open: RwSignal::new(false),
            help_open: RwSignal::new(false),
            health: RwSignal::new(ConnectionHealth::Unconfigured),
            site_label: RwSignal::new(String::new()),
            table_name: RwSignal::new("drugitems".to_string()),
            ignore_columns: RwSignal::new(Vec::new()),
            snapshot: RwSignal::new(None),
            report: RwSignal::new(None),
            comparing: RwSignal::new(false),
            compare_error: RwSignal::new(None),
            last_action: RwSignal::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

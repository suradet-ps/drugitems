//! Data model shared across the DrugItems boundary.
//!
//! Both the Rust backend and the wasm frontend use these types directly
//! (serde, `camelCase` over the wire). "Expected" always means *the value
//! from the snapshot file* (the source of truth); "actual" means *the value
//! currently in the MySQL database*.

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A single cell value as read from one of the two sources.
///
/// Both loaders normalize their native cells into this enum so the engine
/// is source-agnostic: the `.xls` reader produces numbers/strings/dates
/// directly, while the database reader always produces [`RawCell::Text`]
/// or [`RawCell::Null`] (every MySQL value is `CAST(... AS CHAR)` first).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum RawCell {
    /// SQL `NULL` or an empty Excel cell (indistinguishable in the export).
    Null,
    /// A boolean value.
    Bool(bool),
    /// A numeric cell (Excel number, or a numeric string parsed by the
    /// engine when the column is numeric).
    Num(f64),
    /// A text cell.
    Text(String),
    /// A real date/time value (from a date-styled Excel cell).
    DateTime(NaiveDateTime),
}

/// A single row of cells keyed by column name (only the columns that were
/// present on that side).
pub type RowCells = BTreeMap<String, RawCell>;

/// Column value class, derived from the MySQL `DATA_TYPE` of the database
/// column (or [`ColumnKind::Text`] for snapshot-only columns).
///
/// The kind drives canonicalization so that the same *logical* value stored
/// differently by the two tools still compares equal (e.g. `18.8500` vs
/// `18.85`, an Excel date serial `44961.6197...` vs `2023-01-31 14:52:00`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnKind {
    /// Strings, enums, sets, blobs. Compared as canonical text.
    Text,
    /// INT/DECIMAL/FLOAT/DOUBLE/… Compared as numbers.
    Numeric,
    /// DATE columns. Compared as calendar dates.
    Date,
    /// DATETIME/TIMESTAMP columns. Compared as instants.
    DateTime,
    /// TIME columns. Compared as clock times.
    Time,
}

/// One column of the database table with its value class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMeta {
    /// Column name.
    pub name: String,
    /// Value class driving canonicalization.
    pub kind: ColumnKind,
}

/// Everything the engine needs to know about the database side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTable {
    /// Database columns (name + kind), in `information_schema` order.
    pub columns: Vec<ColumnMeta>,
    /// Rows keyed by the `icode` column value.
    pub rows: BTreeMap<String, RowCells>,
}

/// Everything the engine needs to know about the snapshot side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotTable {
    /// File name shown in the UI (never the full path).
    pub file_name: Option<String>,
    /// Snapshot column names, in file order.
    pub columns: Vec<String>,
    /// Rows keyed by the `icode` column value.
    pub rows: BTreeMap<String, RowCells>,
}

/// Status of a single drug item after the comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeStatus {
    /// Present on both sides with identical values.
    Unchanged,
    /// Present on both sides but at least one column differs.
    Changed,
    /// Only in the database (absent from the snapshot) - added after the
    /// snapshot was taken.
    AddedInDb,
    /// Only in the snapshot (absent from the database) - deleted after the
    /// snapshot was taken.
    MissingInDb,
}

/// One changed cell, with both sides resolved to display text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDiff {
    /// Column name.
    pub column: String,
    /// Column value class.
    pub kind: ColumnKind,
    /// Value from the snapshot (source of truth).
    pub expected: CellView,
    /// Value currently in the database.
    pub actual: CellView,
}

/// A display-ready cell value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellView {
    /// Human-readable text (dates resolved from Excel serials, numbers
    /// normalized).
    pub text: String,
    /// True when the cell was empty/NULL - the UI renders a dash.
    pub empty: bool,
}

/// One drug item row result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowDiff {
    /// `icode` (the row key).
    pub code: String,
    /// Value of the `name` column on the present side, when available - a
    /// human-readable handle for added/missing rows.
    pub name: Option<String>,
    /// Row status.
    pub status: ChangeStatus,
    /// Changed cells; empty for `Unchanged`, `AddedInDb`, `MissingInDb`.
    pub changes: Vec<ColumnDiff>,
}

/// Per-column change tally - "which columns were touched, and how often".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnCount {
    /// Column name.
    pub column: String,
    /// Column value class.
    pub kind: ColumnKind,
    /// Number of rows whose value in this column differs.
    pub changed_rows: usize,
}

/// Row-count summary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    /// Rows identical on both sides.
    pub unchanged: usize,
    /// Rows present on both sides with at least one changed column.
    pub changed: usize,
    /// Rows only in the database.
    pub added_in_db: usize,
    /// Rows only in the snapshot.
    pub missing_in_db: usize,
}

/// The full result of a comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareReport {
    /// Local wall-clock timestamp of the comparison (ISO 8601).
    pub generated_at: String,
    /// Snapshot file name, if loaded.
    pub snapshot_file: Option<String>,
    /// Number of data rows in the snapshot.
    pub snapshot_rows: usize,
    /// Number of rows returned by the database query.
    pub db_rows: usize,
    /// `database.table` the comparison ran against.
    pub db_table: Option<String>,
    /// Row-count summary.
    pub counts: Counts,
    /// Human-readable (Thai) structural observations: columns present on
    /// only one side, ignored columns, etc.
    pub schema_notes: Vec<String>,
    /// Columns the operator chose to skip in the comparison.
    pub ignored_columns: Vec<String>,
    /// Changed columns sorted by descending change count.
    pub changed_columns: Vec<ColumnCount>,
    /// Per-row results for rows that are not unchanged.
    pub rows: Vec<RowDiff>,
}

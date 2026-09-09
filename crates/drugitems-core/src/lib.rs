//! DrugItems core: the pure, testable heart of the app.
//!
//! The app answers one question: *has anything in the MySQL `drugitems`
//! table drifted from the `.xls` snapshot that was taken earlier?* The
//! snapshot is the source of truth - every column, every value.
//!
//! This crate holds the model ([`model`]) and the comparison engine
//! ([`engine`]) with no I/O of its own, plus Excel-serial date math
//! ([`serial`]). It is used on both sides of the Tauri boundary (the
//! wasm frontend reuses the report types directly).

pub mod engine;
pub mod model;
pub mod serial;

pub use engine::compare;
pub use model::{
    CellView, ChangeStatus, ColumnCount, ColumnDiff, ColumnKind, ColumnMeta, CompareReport,
    Counts, DbTable, RawCell, RowCells, RowDiff, SnapshotTable,
};

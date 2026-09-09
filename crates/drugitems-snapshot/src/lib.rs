//! DrugItems snapshot loader.
//!
//! Reads the exported spreadsheet (`.xls`/`.xlsx`, via `calamine`) into a
//! [`drugitems_core::SnapshotTable`] - the **source of truth**. The loader
//! is deliberately strict about shape: duplicate columns, missing `icode`
//! column, and duplicate `icode` values are all hard errors rather than
//! silent data corruption.

use std::path::Path;

use calamine::Data;
use drugitems_core::{RawCell, RowCells, SnapshotTable};
use thiserror::Error;

// Brings `sheet_names` / `worksheet_range` into scope for `Sheets`.
use calamine::Reader;

/// The workbook type produced by `open_workbook_auto` for a file path.
type Workbook = calamine::Sheets<std::io::BufReader<std::fs::File>>;

/// Errors produced while loading a snapshot file.
#[derive(Debug, Error)]
pub enum SnapshotError {
    /// The file could not be opened.
    #[error("ไม่สามารถเปิดไฟล์ {path} ได้: {source}")]
    Io {
        /// Path that failed.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The file is not a readable spreadsheet.
    #[error("ไฟล์ {path} ไม่ใช่ไฟล์ Excel ที่อ่านได้: {detail}")]
    Parse {
        /// Path that failed.
        path: String,
        /// Underlying error detail.
        detail: String,
    },
    /// The sheet has no rows at all.
    #[error("ไฟล์ว่างเปล่า - ไม่มีแถวข้อมูล")]
    EmptyFile,
    /// Two columns share the same name.
    #[error("ไฟล์มีชื่อคอลัมน์ซ้ำกัน: {0}")]
    DuplicateColumn(String),
    /// The `icode` column is missing.
    #[error("ไฟล์ไม่มีคอลัมน์ icode - ใช้เป็นคีย์ของแต่ละรายการยา")]
    MissingIcodeColumn,
    /// Two rows share the same `icode`.
    #[error("ไฟล์มีรหัส icode ซ้ำกัน: {0} - ไม่สามารถใช้เป็น snapshot ที่เชื่อถือได้")]
    DuplicateRow(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, SnapshotError>;

/// Load a snapshot spreadsheet into a [`SnapshotTable`].
///
/// The first non-empty sheet is used; the first row is the header; rows are
/// keyed by the `icode` column. Only non-blank header columns are kept.
pub fn load_snapshot(path: &Path) -> Result<SnapshotTable> {
    let mut wb = calamine::open_workbook_auto(path).map_err(|detail| SnapshotError::Parse {
        path: path.display().to_string(),
        detail: detail.to_string(),
    })?;

    let sheet_name = pick_sheet(&mut wb, &path)?;
    let range = wb
        .worksheet_range(&sheet_name)
        .map_err(|detail| SnapshotError::Parse {
            path: path.display().to_string(),
            detail: detail.to_string(),
        })?;

    let mut rows_iter = range.rows();
    let header = rows_iter.next().ok_or(SnapshotError::EmptyFile)?;

    // Header → column index. Blank headers are dropped; duplicates are a
    // hard error (they would corrupt the comparison).
    let mut headers: Vec<(usize, String)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (idx, cell) in header.iter().enumerate() {
        let name = cell_to_text(cell).trim().to_string();
        if name.is_empty() {
            continue;
        }
        if !seen.insert(name.clone()) {
            return Err(SnapshotError::DuplicateColumn(name));
        }
        headers.push((idx, name));
    }

    let code_idx = headers
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case("icode"))
        .map(|(idx, _)| *idx)
        .ok_or(SnapshotError::MissingIcodeColumn)?;

    let columns: Vec<String> = headers.iter().map(|(_, name)| name.clone()).collect();
    let mut rows: std::collections::BTreeMap<String, RowCells> = Default::default();

    for row in rows_iter {
        let code = match row.get(code_idx) {
            Some(cell) => cell_to_text(cell).trim().to_string(),
            None => continue, // trailing blank rows are ignored
        };
        if code.is_empty() {
            continue;
        }
        if rows.contains_key(&code) {
            return Err(SnapshotError::DuplicateRow(code));
        }
        let mut cells: RowCells = Default::default();
        for (idx, name) in &headers {
            if *idx == code_idx {
                continue;
            }
            let cell = match row.get(*idx) {
                Some(cell) => cell,
                None => continue,
            };
            cells.insert(name.clone(), data_to_raw(cell));
        }
        rows.insert(code, cells);
    }

    Ok(SnapshotTable {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string()),
        columns,
        rows,
    })
}

/// Choose which sheet to read: the single sheet when there is one, else the
/// first sheet that actually contains data (a full-column MySQL export may
/// carry an empty placeholder sheet).
fn pick_sheet(wb: &mut Workbook, path: &Path) -> Result<String> {
    let names = wb.sheet_names();
    if names.len() == 1 {
        return Ok(names[0].clone());
    }
    for name in &names {
        if let Ok(range) = wb.worksheet_range(name) {
            if range.rows().next().is_some() {
                return Ok(name.clone());
            }
        }
    }
    let _ = path;
    Err(SnapshotError::EmptyFile)
}

/// Map a raw spreadsheet cell into the shared [`RawCell`] model.
fn data_to_raw(cell: &Data) -> RawCell {
    match cell {
        Data::Empty => RawCell::Null,
        Data::Bool(b) => RawCell::Bool(*b),
        Data::Float(f) => RawCell::Num(*f),
        Data::Int(i) => RawCell::Num(*i as f64),
        Data::String(s) => RawCell::Text(s.clone()),
        Data::DateTime(dt) => match dt.as_datetime() {
            Some(dt) => RawCell::DateTime(dt),
            None => RawCell::Text(String::new()),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => RawCell::Text(s.clone()),
        Data::Error(e) => RawCell::Text(format!("#ERR {e}")),
    }
}

/// Textual form of a cell (used for header cells and the `icode` key).
fn cell_to_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Data::Float(f) => trim_float(*f),
        Data::Int(i) => i.to_string(),
        Data::String(s) => s.clone(),
        Data::DateTime(dt) => dt
            .as_datetime()
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR {e}"),
    }
}

/// Shortest decimal representation of a float (Excel `1000001` must not
/// become `"1000001.0"` when it is the row key).
fn trim_float(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

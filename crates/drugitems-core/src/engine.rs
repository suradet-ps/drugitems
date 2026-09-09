//! The comparison engine.
//!
//! Given the database side ([`DbTable`]) and the snapshot side
//! ([`SnapshotTable`]), produce a [`CompareReport`] classifying every row
//! and every changed cell. The snapshot is the source of truth; the
//! database is the suspect.
//!
//! Comparison is **canonicalization-based**: two values are equal when they
//! represent the same logical value, even if the two tools stored them
//! differently - `18.8500` vs `18.85`, an Excel date serial `44961.6197...`
//! vs `2023-01-31 14:52:30`, a numeric-looking string `"1000001"` vs the
//! Excel number `1000001`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};

use crate::model::*;
use crate::serial;

/// Build the report. `ignore_columns` are excluded from the cell-by-cell
/// comparison (rows are still classified by presence).
pub fn compare(
    db: &DbTable,
    snapshot: &SnapshotTable,
    ignore_columns: &HashSet<String>,
) -> CompareReport {
    let db_names: HashMap<&str, ColumnKind> = db
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c.kind))
        .collect();
    let snap_names: HashSet<&str> = snapshot.columns.iter().map(String::as_str).collect();

    // Columns compared cell-by-cell: present on BOTH sides, not ignored.
    // Kind comes from the database (the snapshot has no type information).
    let compared: Vec<(&str, ColumnKind)> = db
        .columns
        .iter()
        .filter(|c| snap_names.contains(c.name.as_str()) && !ignore_columns.contains(&c.name))
        .map(|c| (c.name.as_str(), c.kind))
        .collect();

    let mut schema_notes: Vec<String> = Vec::new();
    let db_only: Vec<&str> = db_names
        .keys()
        .copied()
        .filter(|n| !snap_names.contains(n))
        .collect();
    if !db_only.is_empty() {
        schema_notes.push(format!(
            "คอลัมน์ในฐานข้อมูลที่ไม่มีในไฟล์ (ไม่ได้นำมาเปรียบเทียบ): {}",
            join_names(&db_only)
        ));
    }
    let snap_only: Vec<&str> = snapshot
        .columns
        .iter()
        .map(String::as_str)
        .filter(|n| !db_names.contains_key(n))
        .collect();
    if !snap_only.is_empty() {
        schema_notes.push(format!(
            "คอลัมน์ในไฟล์ที่ไม่มีในฐานข้อมูล (ไม่ได้นำมาเปรียบเทียบ): {}",
            join_names(&snap_only)
        ));
    }

    // Union of row keys (icode), sorted for deterministic output.
    let mut codes: BTreeSet<&str> = BTreeSet::new();
    codes.extend(db.rows.keys().map(String::as_str));
    codes.extend(snapshot.rows.keys().map(String::as_str));

    let mut counts = Counts::default();
    let mut rows: Vec<RowDiff> = Vec::new();
    let mut col_hits: BTreeMap<&str, (ColumnKind, usize)> = BTreeMap::new();

    for code in codes {
        let db_cells = db.rows.get(code);
        let snap_cells = snapshot.rows.get(code);

        let status = match (db_cells, snap_cells) {
            (Some(_), Some(_)) => ChangeStatus::Unchanged,
            (Some(_), None) => ChangeStatus::AddedInDb,
            (None, Some(_)) => ChangeStatus::MissingInDb,
            (None, None) => unreachable!("code came from the union"),
        };

        let mut changes: Vec<ColumnDiff> = Vec::new();
        if let (Some(db_cells), Some(snap_cells)) = (db_cells, snap_cells) {
            for &(name, kind) in &compared {
                // Only compare cells that exist on BOTH sides. The row-key
                // column (`icode`) is the map key, not a cell: the snapshot
                // loader never carries it, and neither should the database
                // side. Comparing an absent key against a value would flag
                // every row as "changed" on `icode`.
                let (Some(expected), Some(actual)) = (snap_cells.get(name), db_cells.get(name))
                else {
                    continue;
                };
                if !cells_equal(kind, Some(expected), Some(actual)) {
                    changes.push(ColumnDiff {
                        column: name.to_string(),
                        kind,
                        expected: display(Some(expected), kind),
                        actual: display(Some(actual), kind),
                    });
                    col_hits
                        .entry(name)
                        .and_modify(|(_, n)| *n += 1)
                        .or_insert((kind, 1));
                }
            }
        }

        let is_changed = !changes.is_empty();
        let status = if status == ChangeStatus::Unchanged && is_changed {
            ChangeStatus::Changed
        } else {
            status
        };

        match status {
            ChangeStatus::Unchanged => counts.unchanged += 1,
            ChangeStatus::Changed => counts.changed += 1,
            ChangeStatus::AddedInDb => counts.added_in_db += 1,
            ChangeStatus::MissingInDb => counts.missing_in_db += 1,
        }

        if status != ChangeStatus::Unchanged {
            let name = snap_cells
                .and_then(row_name)
                .or_else(|| db_cells.and_then(row_name));
            rows.push(RowDiff {
                code: code.to_string(),
                name,
                status,
                changes,
            });
        }
    }

    let mut changed_columns: Vec<ColumnCount> = col_hits
        .into_iter()
        .map(|(column, (kind, changed_rows))| ColumnCount {
            column: column.to_string(),
            kind,
            changed_rows,
        })
        .collect();
    changed_columns.sort_by(|a, b| {
        b.changed_rows
            .cmp(&a.changed_rows)
            .then_with(|| a.column.cmp(&b.column))
    });

    let mut ignored: Vec<String> = ignore_columns.iter().cloned().collect();
    ignored.sort();

    CompareReport {
        generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        snapshot_file: snapshot.file_name.clone(),
        snapshot_rows: snapshot.rows.len(),
        db_rows: db.rows.len(),
        db_table: None,
        counts,
        schema_notes,
        ignored_columns: ignored,
        changed_columns,
        rows,
    }
}

fn join_names(names: &[&str]) -> String {
    names.join(", ")
}

/// The `name` column value of a row, when present and non-empty - a
/// human-readable handle for added/missing rows.
fn row_name(cells: &RowCells) -> Option<String> {
    match cells.get("name") {
        Some(RawCell::Text(s)) if !s.trim().is_empty() => Some(s.clone()),
        Some(RawCell::Text(s)) if s.trim().is_empty() => None,
        Some(RawCell::Num(v)) => Some(canonical_f64(*v)),
        Some(RawCell::Bool(b)) => Some(if *b { "true" } else { "false" }.to_string()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Canonicalization
// ---------------------------------------------------------------------------

/// Result of turning a cell into a typed value: empty (NULL/blank),
/// a parsed value, or something that does not parse.
enum Parse<T> {
    Empty,
    Value(T),
    Invalid,
}

fn parse_numeric(cell: Option<&RawCell>) -> Parse<f64> {
    match cell {
        None | Some(RawCell::Null) => Parse::Empty,
        Some(RawCell::Text(s)) if text_blank(s) => Parse::Empty,
        Some(RawCell::Num(v)) => Parse::Value(*v),
        Some(RawCell::Bool(b)) => Parse::Value(if *b { 1.0 } else { 0.0 }),
        Some(RawCell::Text(s)) => match s.trim().parse::<f64>() {
            Ok(v) => Parse::Value(v),
            Err(_) => Parse::Invalid,
        },
        Some(RawCell::DateTime(_)) => Parse::Invalid,
    }
}

fn parse_date(cell: Option<&RawCell>) -> Parse<NaiveDate> {
    match cell {
        None | Some(RawCell::Null) => Parse::Empty,
        Some(RawCell::Text(s)) if text_blank(s) => Parse::Empty,
        Some(RawCell::Num(v)) => match serial::serial_to_date(*v) {
            Some(d) => Parse::Value(d),
            None => Parse::Invalid,
        },
        Some(RawCell::DateTime(dt)) => Parse::Value(dt.date()),
        Some(RawCell::Text(s)) => match parse_naive_datetime(s.trim()) {
            Some(dt) => Parse::Value(dt.date()),
            None => Parse::Invalid,
        },
        Some(RawCell::Bool(_)) => Parse::Invalid,
    }
}

fn parse_datetime(cell: Option<&RawCell>) -> Parse<NaiveDateTime> {
    match cell {
        None | Some(RawCell::Null) => Parse::Empty,
        Some(RawCell::Text(s)) if text_blank(s) => Parse::Empty,
        Some(RawCell::Num(v)) => match serial::serial_to_datetime(*v) {
            Some(dt) => Parse::Value(dt),
            None => Parse::Invalid,
        },
        Some(RawCell::DateTime(dt)) => Parse::Value(*dt),
        Some(RawCell::Text(s)) => match parse_naive_datetime(s.trim()) {
            Some(dt) => Parse::Value(dt),
            None => Parse::Invalid,
        },
        Some(RawCell::Bool(_)) => Parse::Invalid,
    }
}

fn parse_time(cell: Option<&RawCell>) -> Parse<NaiveTime> {
    match cell {
        None | Some(RawCell::Null) => Parse::Empty,
        Some(RawCell::Text(s)) if text_blank(s) => Parse::Empty,
        Some(RawCell::Num(v)) => match serial::serial_to_time(*v) {
            Some(t) => Parse::Value(t),
            None => Parse::Invalid,
        },
        Some(RawCell::DateTime(dt)) => Parse::Value(dt.time()),
        Some(RawCell::Text(s)) => {
            let s = s.trim();
            let fmt = |f: &str| NaiveTime::parse_from_str(s, f).ok();
            match fmt("%H:%M:%S%.f").or_else(|| fmt("%H:%M:%S")).or_else(|| fmt("%H:%M")) {
                Some(t) => Parse::Value(t),
                None => Parse::Invalid,
            }
        }
        Some(RawCell::Bool(_)) => Parse::Invalid,
    }
}

/// Canonical text for a cell (used for [`ColumnKind::Text`] columns).
fn canonical_text(cell: Option<&RawCell>) -> Option<String> {
    match cell {
        None | Some(RawCell::Null) => None,
        Some(RawCell::Text(s)) if text_blank(s) => None,
        Some(RawCell::Text(s)) => canonical_number(s).or_else(|| Some(normalize_text(s))),
        Some(RawCell::Num(v)) => Some(canonical_f64(*v)),
        Some(RawCell::Bool(b)) => Some(if *b { "true" } else { "false" }.to_string()),
        Some(RawCell::DateTime(dt)) => Some(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
    }
}

/// Normalize whitespace artifacts introduced by the export round-trip, so
/// the same logical text compares equal even when the tools stored it
/// differently:
///
/// * `CRLF` / lone `CR` → `LF` (the database may keep `\r\n` while Excel
///   normalizes to `\n`),
/// * trailing whitespace (spaces, tabs, newlines) is trimmed - an export
///   often adds or drops a final newline in a text cell.
///
/// Leading whitespace is preserved (it can be meaningful in multi-line
/// dosing instructions).
fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end()
        .to_string()
}

fn text_blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn cells_equal(kind: ColumnKind, expected: Option<&RawCell>, actual: Option<&RawCell>) -> bool {
    match kind {
        ColumnKind::Text => canonical_text(expected) == canonical_text(actual),
        ColumnKind::Numeric => match (parse_numeric(expected), parse_numeric(actual)) {
            (Parse::Empty, Parse::Empty) => true,
            (Parse::Value(a), Parse::Value(b)) => numeric_eq(a, b),
            _ => false,
        },
        ColumnKind::Date => match (parse_date(expected), parse_date(actual)) {
            (Parse::Empty, Parse::Empty) => true,
            (Parse::Value(a), Parse::Value(b)) => a == b,
            _ => false,
        },
        ColumnKind::DateTime => match (parse_datetime(expected), parse_datetime(actual)) {
            (Parse::Empty, Parse::Empty) => true,
            (Parse::Value(a), Parse::Value(b)) => a == b,
            _ => false,
        },
        ColumnKind::Time => match (parse_time(expected), parse_time(actual)) {
            (Parse::Empty, Parse::Empty) => true,
            (Parse::Value(a), Parse::Value(b)) => a == b,
            _ => false,
        },
    }
}

/// Numeric equality with a small relative tolerance, so values that round
/// differently through the two tools (float vs decimal vs text) still
/// compare equal instead of raising false alarms.
fn numeric_eq(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

/// Shortest round-trip decimal representation of a float; integers are
/// printed without a decimal point.
fn canonical_f64(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// Canonical form of a numeric-looking string: `"1000001"` → `"1000001"`,
/// `"18.8500"` → `"18.85"`, `"57"` → `"57"`. Non-numeric strings return
/// `None` (they are compared verbatim).
fn canonical_number(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(i) = t.parse::<i64>() {
        return Some(i.to_string());
    }
    match t.parse::<f64>() {
        Ok(v) if v.is_finite() => Some(canonical_f64(v)),
        _ => None,
    }
}

/// Parse a date/time text produced by the database or the export tool.
///
/// Accepts ISO `YYYY-MM-DD[ HH:MM[:SS[.f]]]` (and `YYYY/MM/DD`) and shifts
/// Buddhist-Era years (≥ 2500) back to Christian Era, since HOSxP sites may
/// store either era.
fn parse_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    const DATETIME_FORMATS: [&str; 4] = [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
    ];
    const DATE_FORMATS: [&str; 2] = ["%Y-%m-%d", "%Y/%m/%d"];
    let dt = DATETIME_FORMATS
        .iter()
        .find_map(|fmt| NaiveDateTime::parse_from_str(s, fmt).ok())
        .or_else(|| {
            DATE_FORMATS
                .iter()
                .find_map(|fmt| NaiveDate::parse_from_str(s, fmt).ok())
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })?;
    if dt.year() >= 2500 {
        Some(dt.checked_sub_months(chrono::Months::new(543 * 12))?)
    } else {
        Some(dt)
    }
}

/// Display a cell for the UI, resolving it per column kind (Excel serials
/// become real dates, numbers normalize, blanks become an empty marker).
fn display(cell: Option<&RawCell>, kind: ColumnKind) -> CellView {
    let empty = CellView {
        text: String::new(),
        empty: true,
    };
    let filled = |text: String| CellView {
        text,
        empty: false,
    };
    match kind {
        ColumnKind::Text => match canonical_text(cell) {
            None => empty,
            Some(text) => filled(text),
        },
        ColumnKind::Numeric => match parse_numeric(cell) {
            Parse::Empty => empty,
            Parse::Value(v) => filled(canonical_f64(v)),
            Parse::Invalid => filled(raw_text(cell)),
        },
        ColumnKind::Date => match parse_date(cell) {
            Parse::Empty => empty,
            Parse::Value(d) => filled(d.format("%Y-%m-%d").to_string()),
            Parse::Invalid => filled(raw_text(cell)),
        },
        ColumnKind::DateTime => match parse_datetime(cell) {
            Parse::Empty => empty,
            Parse::Value(dt) => filled(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            Parse::Invalid => filled(raw_text(cell)),
        },
        ColumnKind::Time => match parse_time(cell) {
            Parse::Empty => empty,
            Parse::Value(t) => filled(t.format("%H:%M:%S").to_string()),
            Parse::Invalid => filled(raw_text(cell)),
        },
    }
}

fn raw_text(cell: Option<&RawCell>) -> String {
    match cell {
        None | Some(RawCell::Null) => String::new(),
        Some(RawCell::Bool(b)) => if *b { "true" } else { "false" }.to_string(),
        Some(RawCell::Num(v)) => canonical_f64(*v),
        Some(RawCell::Text(s)) => s.clone(),
        Some(RawCell::DateTime(dt)) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db_table(rows: Vec<(&str, RowCells)>) -> DbTable {
        DbTable {
            columns: vec![
                ColumnMeta {
                    name: "icode".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "name".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "unitprice".into(),
                    kind: ColumnKind::Numeric,
                },
                ColumnMeta {
                    name: "lastupdatestdprice".into(),
                    kind: ColumnKind::DateTime,
                },
                ColumnMeta {
                    name: "istatus".into(),
                    kind: ColumnKind::Text,
                },
            ],
            rows: rows
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    fn snapshot_table(rows: Vec<(&str, RowCells)>) -> SnapshotTable {
        SnapshotTable {
            file_name: Some("snapshot.xls".into()),
            columns: vec![
                "icode".into(),
                "name".into(),
                "unitprice".into(),
                "lastupdatestdprice".into(),
                "istatus".into(),
            ],
            rows: rows
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    fn row(cells: &[(&str, RawCell)]) -> RowCells {
        cells
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn text(s: &str) -> RawCell {
        RawCell::Text(s.into())
    }

    fn num(v: f64) -> RawCell {
        RawCell::Num(v)
    }

    fn ignore(none: &[&str]) -> HashSet<String> {
        none.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn identical_tables_are_all_unchanged() {
        let cells = row(&[
            ("icode", text("1000001")),
            ("name", text("TRIAMCINOLONE")),
            ("unitprice", num(18.85)),
            ("lastupdatestdprice", text("2023-01-31 14:52:30")),
            ("istatus", text("N")),
        ]);
        let db = db_table(vec![("1000001", cells.clone())]);
        let snap = snapshot_table(vec![("1000001", cells)]);
        let report = compare(&db, &snap, &ignore(&[]));
        assert_eq!(report.counts.unchanged, 1);
        assert_eq!(report.counts.changed, 0);
        assert!(report.rows.is_empty());
    }

    #[test]
    fn numeric_representation_differences_are_equal() {
        // DB stores DECIMAL 18.8500 (as text after CAST); snapshot has 18.85.
        let db_cells = row(&[
            ("icode", text("1000001")),
            ("name", text("X")),
            ("unitprice", text("18.8500")),
            ("lastupdatestdprice", text("2023-02-04 14:52:30")),
            ("istatus", text("N")),
        ]);
        let snap_cells = row(&[
            ("icode", num(1000001.0)),
            ("name", text("X")),
            ("unitprice", num(18.85)),
            ("lastupdatestdprice", num(44961.619791666664)),
            ("istatus", text("N")),
        ]);
        let report = compare(&db_table(vec![("1000001", db_cells)]), &snapshot_table(vec![("1000001", snap_cells)]), &ignore(&[]));
        assert_eq!(report.counts.unchanged, 1, "must not raise a false alarm");
    }

    #[test]
    fn numeric_text_with_leading_zeros_collapses() {
        // Documented trade-off: a numeric-looking text "010" and the number
        // 10 compare equal. Rare in HOSxP icodes but a known blind spot.
        let db_cells = row(&[("icode", text("010")), ("name", text("X"))]);
        let snap_cells = row(&[("icode", num(10.0)), ("name", text("X"))]);
        let report = compare(&db_table(vec![("010", db_cells)]), &snapshot_table(vec![("010", snap_cells)]), &ignore(&[]));
        assert_eq!(report.counts.unchanged, 1);
    }

    #[test]
    fn real_value_change_is_reported_with_both_sides() {
        let db_cells = row(&[
            ("icode", text("1000001")),
            ("name", text("X")),
            ("unitprice", text("19.00")),
            ("lastupdatestdprice", text("2023-02-04 14:52:30")),
            ("istatus", text("N")),
        ]);
        let snap_cells = row(&[
            ("icode", num(1000001.0)),
            ("name", text("X")),
            ("unitprice", num(18.85)),
            ("lastupdatestdprice", num(44961.619791666664)),
            ("istatus", text("N")),
        ]);
        let report = compare(&db_table(vec![("1000001", db_cells)]), &snapshot_table(vec![("1000001", snap_cells)]), &ignore(&[]));
        assert_eq!(report.counts.changed, 1);
        assert_eq!(report.rows.len(), 1);
        let diff = &report.rows[0];
        assert_eq!(diff.status, ChangeStatus::Changed);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].column, "unitprice");
        assert_eq!(diff.changes[0].expected.text, "18.85");
        assert_eq!(diff.changes[0].actual.text, "19");
        assert_eq!(report.changed_columns.len(), 1);
        assert_eq!(report.changed_columns[0].changed_rows, 1);
    }

    #[test]
    fn added_and_missing_rows_are_classified() {
        let db_cells = row(&[("icode", text("1000001")), ("name", text("X"))]);
        let snap_cells = row(&[("icode", text("1000001")), ("name", text("X"))]);
        let new_cells = row(&[("icode", text("9999999")), ("name", text("ใหม่"))]);
        let gone_cells = row(&[("icode", text("8888888")), ("name", text("หายไป"))]);
        let report = compare(
            &db_table(vec![
                ("1000001", db_cells),
                ("9999999", new_cells),
            ]),
            &snapshot_table(vec![
                ("1000001", snap_cells),
                ("8888888", gone_cells),
            ]),
            &ignore(&[]),
        );
        assert_eq!(report.counts.unchanged, 1);
        assert_eq!(report.counts.added_in_db, 1);
        assert_eq!(report.counts.missing_in_db, 1);
        assert!(report
            .rows
            .iter()
            .any(|r| r.status == ChangeStatus::AddedInDb && r.code == "9999999" && r.name.as_deref() == Some("ใหม่")));
        assert!(report
            .rows
            .iter()
            .any(|r| r.status == ChangeStatus::MissingInDb && r.code == "8888888"));
    }

    #[test]
    fn null_and_blank_are_equivalent() {
        let db_cells = row(&[("icode", text("1")), ("name", RawCell::Null)]);
        let snap_cells = row(&[("icode", text("1")), ("name", RawCell::Text("".into()))]);
        let report = compare(&db_table(vec![("1", db_cells)]), &snapshot_table(vec![("1", snap_cells)]), &ignore(&[]));
        assert_eq!(report.counts.unchanged, 1);
    }

    #[test]
    fn ignored_columns_are_not_compared() {
        let db_cells = row(&[("icode", text("1")), ("name", text("A")), ("unitprice", text("99"))]);
        let snap_cells = row(&[("icode", text("1")), ("name", text("A")), ("unitprice", num(1.0))]);
        let report = compare(
            &db_table(vec![("1", db_cells)]),
            &snapshot_table(vec![("1", snap_cells)]),
            &ignore(&["unitprice"]),
        );
        assert_eq!(report.counts.unchanged, 1);
        assert_eq!(report.ignored_columns, vec!["unitprice"]);
    }

    #[test]
    fn missing_columns_on_either_side_are_schema_notes_only() {
        let db = DbTable {
            columns: vec![
                ColumnMeta {
                    name: "icode".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "name".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "drugaccount".into(),
                    kind: ColumnKind::Text,
                },
            ],
            rows: [(
                "1".to_string(),
                row(&[
                    ("icode", text("1")),
                    ("name", text("A")),
                    ("drugaccount", text("01")),
                ]),
            )]
            .into_iter()
            .collect(),
        };
        let snap = SnapshotTable {
            file_name: Some("s.xls".into()),
            columns: vec!["icode".into(), "name".into(), "extra_col".into()],
            rows: [(
                "1".to_string(),
                row(&[
                    ("icode", text("1")),
                    ("name", text("A")),
                    ("extra_col", text("z")),
                ]),
            )]
            .into_iter()
            .collect(),
        };
        let report = compare(&db, &snap, &ignore(&[]));
        assert_eq!(report.counts.unchanged, 1, "no cell comparison for non-shared columns");
        assert!(report.schema_notes.iter().any(|n| n.contains("drugaccount")));
        assert!(report.schema_notes.iter().any(|n| n.contains("extra_col")));
    }

    #[test]
    fn row_key_icode_is_never_compared() {
        // The snapshot loader keys rows by icode and never carries it as a
        // cell. If the database side does carry it, the engine must still
        // not flag every row as "changed" on the icode column.
        let db_cells = row(&[
            ("icode", text("1000001")),
            ("name", text("X")),
            ("unitprice", text("18.85")),
            ("lastupdatestdprice", text("2023-02-04 14:52:30")),
            ("istatus", text("N")),
        ]);
        let snap_cells = row(&[
            ("name", text("X")),
            ("unitprice", num(18.85)),
            ("lastupdatestdprice", num(44961.619791666664)),
            ("istatus", text("N")),
        ]);
        let report = compare(
            &db_table(vec![("1000001", db_cells)]),
            &snapshot_table(vec![("1000001", snap_cells)]),
            &ignore(&[]),
        );
        assert_eq!(report.counts.unchanged, 1, "icode must not be compared");
        assert!(
            report.rows.iter().all(|r| r.changes.is_empty()),
            "no column diff may be raised for a row whose only difference is the key"
        );
    }

    fn text_table(rows: Vec<(&str, RowCells)>) -> DbTable {
        DbTable {
            columns: vec![
                ColumnMeta {
                    name: "icode".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "name".into(),
                    kind: ColumnKind::Text,
                },
                ColumnMeta {
                    name: "show_notify_text".into(),
                    kind: ColumnKind::Text,
                },
            ],
            rows: rows
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    fn text_snapshot(rows: Vec<(&str, RowCells)>) -> SnapshotTable {
        SnapshotTable {
            file_name: Some("s.xls".into()),
            columns: vec!["icode".into(), "name".into(), "show_notify_text".into()],
            rows: rows
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    #[test]
    fn text_line_endings_and_trailing_newline_are_equal() {
        // show_notify_text-style multi-line cell: DB keeps CRLF + a trailing
        // newline, Excel normalizes to LF without the trailing newline.
        let db_cells = row(&[
            ("name", text("X")),
            ("show_notify_text", text("* คำแนะนำ\r\nบรรทัด 2\n")),
        ]);
        let snap_cells = row(&[
            ("name", text("X")),
            ("show_notify_text", text("* คำแนะนำ\nบรรทัด 2")),
        ]);
        let report = compare(
            &text_table(vec![("1000004", db_cells)]),
            &text_snapshot(vec![("1000004", snap_cells)]),
            &ignore(&[]),
        );
        assert_eq!(report.counts.unchanged, 1, "whitespace artifacts must not be reported");
    }

    #[test]
    fn real_text_change_is_still_reported() {
        // Normalization must not hide a genuine edit in the middle.
        let db_cells = row(&[
            ("name", text("X")),
            ("show_notify_text", text("* คำแนะนำ\nแบ่งให้วันละ 3 ครั้ง")),
        ]);
        let snap_cells = row(&[
            ("name", text("X")),
            ("show_notify_text", text("* คำแนะนำ\nแบ่งให้วันละ 2 ครั้ง")),
        ]);
        let report = compare(
            &text_table(vec![("1000004", db_cells)]),
            &text_snapshot(vec![("1000004", snap_cells)]),
            &ignore(&[]),
        );
        assert_eq!(report.counts.changed, 1);
        let diff = &report.rows[0].changes[0];
        assert_eq!(diff.column, "show_notify_text");
        assert_eq!(diff.expected.text, "* คำแนะนำ\nแบ่งให้วันละ 2 ครั้ง");
        assert_eq!(diff.actual.text, "* คำแนะนำ\nแบ่งให้วันละ 3 ครั้ง");
    }
}

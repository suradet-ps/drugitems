//! CSV export of a comparison report - the operator's printable/archivable
//! record of every difference.

use drugitems_core::{ChangeStatus, CompareReport};

/// Serialize the report's non-unchanged rows as CSV.
///
/// Rows with changed cells emit one line per changed column; added/missing
/// rows emit a single line with an empty column section.
pub fn build_csv(report: &CompareReport) -> String {
    let mut out = String::new();
    out.push_str("status,icode,name,column,expected,actual\n");
    for row in &report.rows {
        let status = match row.status {
            ChangeStatus::Unchanged => "ตรงกัน",
            ChangeStatus::Changed => "ต่าง",
            ChangeStatus::AddedInDb => "ใหม่ในฐานข้อมูล",
            ChangeStatus::MissingInDb => "หายจากฐานข้อมูล",
        };
        let code = esc(&row.code);
        let name = esc(row.name.as_deref().unwrap_or(""));
        if row.changes.is_empty() {
            out.push_str(&format!("{status},{code},{name},,,\n"));
        } else {
            for change in &row.changes {
                out.push_str(&format!(
                    "{status},{code},{name},{},{},{}\n",
                    esc(&change.column),
                    esc(&change.expected.text),
                    esc(&change.actual.text)
                ));
            }
        }
    }
    out
}

/// Quote a CSV field when it contains a comma, quote, or newline.
fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drugitems_core::{
        CellView, ColumnCount, ColumnDiff, ColumnKind, Counts, RowDiff,
    };

    fn sample_report() -> CompareReport {
        CompareReport {
            generated_at: "2026-09-08 12:00:00".into(),
            snapshot_file: Some("drugiterms-20260908.xls".into()),
            snapshot_rows: 10,
            db_rows: 10,
            db_table: Some("hos.drugitems".into()),
            counts: Counts {
                unchanged: 8,
                changed: 1,
                added_in_db: 1,
                missing_in_db: 0,
            },
            schema_notes: vec![],
            ignored_columns: vec![],
            changed_columns: vec![ColumnCount {
                column: "unitprice".into(),
                kind: ColumnKind::Numeric,
                changed_rows: 1,
            }],
            rows: vec![
                RowDiff {
                    code: "1000001".into(),
                    name: Some("PARACETAMOL".into()),
                    status: ChangeStatus::Changed,
                    changes: vec![ColumnDiff {
                        column: "unitprice".into(),
                        kind: ColumnKind::Numeric,
                        expected: CellView {
                            text: "18.85".into(),
                            empty: false,
                        },
                        actual: CellView {
                            text: "19".into(),
                            empty: false,
                        },
                    }],
                },
                RowDiff {
                    code: "9999999".into(),
                    name: Some("ยาใหม่, ชนิดพิเศษ".into()),
                    status: ChangeStatus::AddedInDb,
                    changes: vec![],
                },
            ],
        }
    }

    #[test]
    fn csv_escapes_commas_and_quotes() {
        let csv = build_csv(&sample_report());
        assert!(csv.starts_with("status,icode,name,column,expected,actual\n"));
        assert!(csv.contains("1000001,PARACETAMOL,unitprice,18.85,19"));
        assert!(csv.contains("9999999,\"ยาใหม่, ชนิดพิเศษ\",,,"));
    }
}

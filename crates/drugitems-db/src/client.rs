//! MySQL client for DrugItems.

use std::collections::BTreeMap;
use std::time::Duration;

use drugitems_core::{ColumnKind, ColumnMeta, DbTable, RawCell, RowCells};
use secrecy::ExposeSecret;
use sqlx::Row;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use tracing::info;

use crate::config::DbConfig;
use crate::error::{Error, Result};
use crate::readonly::assert_read_only;

/// Introspected schema entry: column metadata plus whether the column is
/// binary (which must be read as HEX rather than CAST AS CHAR).
type SchemaEntry = (ColumnMeta, bool);

/// A connected MySQL/MariaDB client (a cloneable pool handle).
#[derive(Clone)]
pub struct DbClient {
    pool: MySqlPool,
}

impl DbClient {
    /// Connect to the configured server.
    pub async fn connect(cfg: DbConfig) -> Result<Self> {
        let options = MySqlConnectOptions::new()
            .host(&cfg.host)
            .port(cfg.port)
            .database(&cfg.database)
            .username(&cfg.user)
            .password(cfg.password.expose_secret());
        let pool = MySqlPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await
            .map_err(|source| Error::Connect {
                host: cfg.host.clone(),
                port: cfg.port,
                database: cfg.database.clone(),
                source,
            })?;
        Ok(Self { pool })
    }

    /// Close the pool.
    pub async fn disconnect(&self) {
        self.pool.close().await;
    }

    /// A `SELECT 1` smoke test.
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Read the columns of `table` from `information_schema`, in ordinal
    /// order. Returns [`Error::NotFound`] when the table has no columns.
    pub async fn table_schema(&self, table: &str) -> Result<Vec<SchemaEntry>> {
        const SCHEMA_SQL: &str = "SELECT COLUMN_NAME, DATA_TYPE FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION";
        let rows = sqlx::query(SCHEMA_SQL)
            .bind(table)
            .fetch_all(&self.pool)
            .await?;
        if rows.is_empty() {
            return Err(Error::NotFound(format!("table {table} not found")));
        }
        let mut schema: Vec<SchemaEntry> = Vec::with_capacity(rows.len());
        for row in rows {
            let name: String = row.try_get(0).map_err(|e| Error::RowShape(e.to_string()))?;
            let data_type: String = row.try_get(1).map_err(|e| Error::RowShape(e.to_string()))?;
            let binary = is_binary_type(&data_type);
            let kind = data_kind(&data_type);
            schema.push((ColumnMeta { name, kind }, binary));
        }
        Ok(schema)
    }

    /// Read the full contents of `table` into a [`DbTable`], keyed by the
    /// `icode` column. Every value is `CAST(... AS CHAR)` (binary columns
    /// become hex) so the result is uniformly [`RawCell::Text`]/[`RawCell::Null`]
    /// and the compare engine can canonicalize it per column kind.
    pub async fn load_table(&self, table: &str) -> Result<DbTable> {
        let schema = self.table_schema(table).await?;
        if !schema
            .iter()
            .any(|(m, _)| m.name.eq_ignore_ascii_case("icode"))
        {
            return Err(Error::NotFound(format!(
                "table {table} has no icode column to key rows on"
            )));
        }

        let table_q = quote_ident(table);
        let code_col = schema
            .iter()
            .position(|(m, _)| m.name.eq_ignore_ascii_case("icode"))
            .expect("invariant: icode presence checked above");
        let exprs: Vec<String> = schema
            .iter()
            .map(|(meta, binary)| {
                let col = quote_ident(&meta.name);
                if *binary {
                    format!("LOWER(HEX({col}))")
                } else {
                    format!("CAST({col} AS CHAR)")
                }
            })
            .collect();
        let stmt = format!("SELECT {} FROM {table_q}", exprs.join(", "));
        assert_read_only(&stmt)?;
        info!(table = %table, "reading full table for comparison");

        let rows = sqlx::query(&stmt).fetch_all(&self.pool).await?;
        let mut out: BTreeMap<String, RowCells> = BTreeMap::new();
        for row in rows {
            // `icode` is the row key - like the snapshot loader, it is not
            // carried as a cell so the engine never compares it.
            let code: Option<String> = row
                .try_get(code_col)
                .map_err(|e| Error::RowShape(e.to_string()))?;
            let code = code.unwrap_or_default();
            if code.is_empty() {
                continue; // a blank key cannot identify a row
            }
            let mut cells: RowCells = BTreeMap::new();
            for (i, (meta, _)) in schema.iter().enumerate() {
                if i == code_col {
                    continue;
                }
                let value: Option<String> =
                    row.try_get(i).map_err(|e| Error::RowShape(e.to_string()))?;
                cells.insert(
                    meta.name.clone(),
                    match value {
                        Some(text) => RawCell::Text(text),
                        None => RawCell::Null,
                    },
                );
            }
            out.insert(code, cells);
        }

        Ok(DbTable {
            columns: schema.into_iter().map(|(meta, _)| meta).collect(),
            rows: out,
        })
    }
}

/// Backtick-quote an identifier, escaping any embedded backticks.
fn quote_ident(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

/// Whether a MySQL data type stores raw bytes that need HEX reading.
fn is_binary_type(data_type: &str) -> bool {
    let t = data_type.to_lowercase();
    t.contains("blob") || t.contains("binary") || t.contains("geometry")
}

/// Map a MySQL `DATA_TYPE` to a compare [`ColumnKind`].
fn data_kind(data_type: &str) -> ColumnKind {
    match data_type.to_lowercase().as_str() {
        "tinyint" | "smallint" | "mediumint" | "int" | "integer" | "bigint" | "year" | "float"
        | "double" | "decimal" | "numeric" | "real" | "bit" => ColumnKind::Numeric,
        "date" => ColumnKind::Date,
        "datetime" | "timestamp" => ColumnKind::DateTime,
        "time" => ColumnKind::Time,
        _ => ColumnKind::Text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_mapping_covers_hosxp_types() {
        assert_eq!(data_kind("varchar"), ColumnKind::Text);
        assert_eq!(data_kind("TEXT"), ColumnKind::Text);
        assert_eq!(data_kind("enum"), ColumnKind::Text);
        assert_eq!(data_kind("decimal"), ColumnKind::Numeric);
        assert_eq!(data_kind("int"), ColumnKind::Numeric);
        assert_eq!(data_kind("double"), ColumnKind::Numeric);
        assert_eq!(data_kind("date"), ColumnKind::Date);
        assert_eq!(data_kind("datetime"), ColumnKind::DateTime);
        assert_eq!(data_kind("timestamp"), ColumnKind::DateTime);
        assert_eq!(data_kind("time"), ColumnKind::Time);
    }

    #[test]
    fn binary_type_detection() {
        assert!(is_binary_type("blob"));
        assert!(is_binary_type("longblob"));
        assert!(is_binary_type("varbinary"));
        assert!(!is_binary_type("varchar"));
        assert!(!is_binary_type("text"));
    }

    #[test]
    fn quote_ident_escapes_backticks() {
        assert_eq!(quote_ident("drugitems"), "`drugitems`");
        assert_eq!(quote_ident("a`b"), "`a``b`");
    }

    #[test]
    fn generated_select_passes_read_only_guard() {
        let stmt = "SELECT CAST(`icode` AS CHAR), CAST(`name` AS CHAR) FROM `drugitems`";
        assert!(assert_read_only(stmt).is_ok());
    }
}

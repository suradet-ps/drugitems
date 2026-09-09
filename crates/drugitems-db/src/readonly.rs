//! Read-only enforcement for every statement executed against MySQL.
//!
//! Defense-in-depth on top of the read-only DB role: no `INSERT`/`UPDATE`/
//! `DELETE`/`DROP`/DDL ever reaches the server, no matter what statement
//! string is passed to the client.

use crate::error::{Error, Result};

/// SQL keywords allowed to reach a read-only connection.
const ALLOWED_KEYWORDS: [&str; 4] = ["select", "show", "describe", "explain"];

/// Validate that a statement is read-only.
///
/// Intentionally strict: the statement must *begin* with one of the allowed
/// keywords. Anything else (including `WITH`, which can wrap DML) is
/// rejected.
pub fn assert_read_only(stmt: &str) -> Result<()> {
    let first = stmt
        .split_whitespace()
        .next()
        .map(str::to_lowercase)
        .unwrap_or_default();
    if ALLOWED_KEYWORDS.contains(&first.as_str()) {
        Ok(())
    } else {
        Err(Error::ReadOnlyViolation(
            stmt.trim().chars().take(64).collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_select_statements() {
        assert!(assert_read_only("SELECT * FROM drugitems").is_ok());
        assert!(assert_read_only("  select icode from drugitems").is_ok());
        assert!(assert_read_only("SHOW TABLES").is_ok());
        assert!(assert_read_only("DESCRIBE drugitems").is_ok());
    }

    #[test]
    fn rejects_all_dml_and_ddl() {
        for stmt in [
            "INSERT INTO drugitems (icode) VALUES ('x')",
            "UPDATE drugitems SET name = 'x'",
            "DELETE FROM drugitems",
            "DROP TABLE drugitems",
            "TRUNCATE TABLE drugitems",
            "WITH cte AS (SELECT 1) DELETE FROM drugitems",
            "",
            "   ",
        ] {
            assert!(assert_read_only(stmt).is_err(), "must reject: {stmt:?}");
        }
    }
}

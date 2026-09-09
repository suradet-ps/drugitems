//! Connection configuration for the MySQL server.

use secrecy::SecretString;

/// MySQL connection settings (password never logged).
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Hostname or IP.
    pub host: String,
    /// TCP port (default 3306).
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Database user - recommended: a read-only role.
    pub user: String,
    /// Database password, kept in a secret wrapper.
    pub password: SecretString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_carries_fields() {
        let c = DbConfig {
            host: "localhost".into(),
            port: 3306,
            database: "hos".into(),
            user: "ro".into(),
            password: SecretString::from("s3cret"),
        };
        assert_eq!(c.database, "hos");
        assert_eq!(c.user, "ro");
    }
}

//! Error type for the database layer.

use thiserror::Error;

/// Errors produced by the database repository.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to establish the connection pool.
    #[error("failed to connect to MySQL at {host}:{port}/{database}: {source}")]
    Connect {
        /// Host from the config.
        host: String,
        /// Port from the config.
        port: u16,
        /// Database name from the config.
        database: String,
        /// Underlying driver error.
        source: sqlx::Error,
    },

    /// A statement was rejected by the read-only guard.
    #[error("read-only guard rejected statement: {0}")]
    ReadOnlyViolation(String),

    /// Required table or column was missing.
    #[error("{0}")]
    NotFound(String),

    /// A result row did not have the expected shape.
    #[error("unexpected row shape: {0}")]
    RowShape(String),

    /// Any other database/driver failure.
    #[error("MySQL database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

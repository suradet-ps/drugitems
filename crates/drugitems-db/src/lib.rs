//! DrugItems database layer.
//!
//! A `sqlx` MySQL/MariaDB client that reads the full `drugitems` table and
//! its schema. Read-only is enforced twice: the recommended database role
//! is read-only, and every statement is checked by the keyword guard in
//! [`readonly`] before execution.

pub mod client;
pub mod config;
pub mod error;
pub mod readonly;

pub use client::DbClient;
pub use config::DbConfig;
pub use error::{Error, Result};

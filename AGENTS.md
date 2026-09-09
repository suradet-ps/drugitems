# AGENTS.md - DrugItems

## Project Overview

DrugItems is a read-only desktop app that verifies whether the MySQL
`drugitems` (drug master) table still matches an exported spreadsheet
snapshot. The snapshot is the **source of truth** - every column, every
value. The app reads the DB with `SELECT`-only statements, compares it
cell-by-cell, and reports added / missing / changed rows with per-column
before-after values.

Product scope, UX flows, and compare rules live in this file and in
`crates/drugitems-core/src/engine.rs`. The visual design system (colors,
typography, components) lives in **docs/DESIGN.md**. This file covers
agent-facing conventions: stack, build commands, coding rules, and the
data contract.

## Tech Stack

- **Shell:** Tauri 2
- **Frontend:** Leptos 0.8, CSR (compiled to WASM via trunk)
- **Snapshot reader:** `calamine` (.xls/.xlsx)
- **Database access:** `sqlx` MySQL/MariaDB, read-only connection only
- **Credential encryption:** `encryptman` (AES-256-GCM + HKDF), master key
  via `encryptman-keyring` (OS keychain). Never store plaintext credentials.
- **Config files** (platform config dir `%APPDATA%\DrugItems` on Windows):
  `connection.json` (encrypted) + `settings.json` (plain: table name, site
  label, ignored columns, last snapshot file).

## Core Constraints

- **Read-only against MySQL.** No `INSERT`/`UPDATE`/`DELETE`/DDL. Every
  statement must begin with `SELECT`/`SHOW`/`DESCRIBE`/`EXPLAIN` - enforced
  by the keyword guard in `crates/drugitems-db/src/readonly.rs` on top of a
  recommended read-only DB role.
- **Snapshot is the truth.** The "expected" side in every diff is the
  snapshot value; the database is the suspect. Do not "fix" the DB - this
  app only reports.
- **Canonical comparison.** Two values that represent the same logical
  value are equal even if stored differently (18.8500 vs 18.85, Excel date
  serial vs ISO datetime, numeric-looking text vs number, NULL vs blank,
  BE year ≥ 2500 auto-subtracted 543, CRLF/CR vs LF, trailing whitespace
  trimmed in text cells). The row-key column `icode` is never compared -
  it is the map key on both sides. See `engine.rs` for the rules and their
  documented blind spots.
- **Thai-only UI** (mirrors Med Recon). Backend error messages are Thai at
  the command layer; library crates keep English typed errors.

## Data Contract (`drugitems` table - from the sample snapshot)

The provided `drugiterms-20260908.xls` export contains **657 rows × 202
columns** (sheet `export`, header row 1). Key columns:

| Column | Meaning | Compare kind |
|---|---|---|
| `icode` | Row key (PK) | text (identity, not compared) |
| `name`, `ename`, `name_pr`, `name_eng` | drug names | text |
| `strength`, `units` | strength / unit text | text |
| `unitprice`, `stdprice`, `unitcost`, `price2/3`, `ipd_price…` | prices | numeric |
| `lastupdatestdprice` | last price-update stamp | datetime (Excel serial) |
| `dosageform` | dosage form | text |
| `criticalpriority` | critical flag | numeric (0/1) |
| `istatus` | status flag (Y/N/T2/…) | text |
| `drugusage`, `frequency_code`, `time_code`, … | dosing | text |
| `tmt_tp_code`, `tmt_gp_code`, `atc_code`, `sks_*` | mapping codes | text |
| `last_update` | row update timestamp | datetime |

There is **no single source of truth for column types except the live
schema**: `drugitems-db` introspects `information_schema.COLUMNS` and maps
`DATA_TYPE` → `ColumnKind` (see `client.rs::data_kind`). Verify against the
live schema before relying on any column's type.

## Development Workflow

- Build / run:
  ```
  rustup target add wasm32-unknown-unknown
  cargo install trunk
  cargo tauri dev          # dev (trunk serve :1420)
  cargo tauri build       # release
  ```
- Checks (no DB needed):
  ```
  cargo check -p drugitems-core -p drugitems-snapshot -p drugitems-db \
              -p drugitems-config -p drugitems-bridge -p drugitems-app
  cargo check -p drugitems-frontend --target wasm32-unknown-unknown
  cargo test -p drugitems-core -p drugitems-db -p drugitems-app -p drugitems-config
  ```
- Useful dev utility: `cargo run -p drugitems-snapshot --example parse -- file.xls`

## Coding Rules (see Med Recon AGENTS-RUST for the full baseline)

- `edition = "2024"`; shared versions in root `[workspace.dependencies]`.
- Library crates: typed `thiserror` enums, no `anyhow`. The app crate uses
  a typed `CommandError { kind, message }` (Thai message for the UI).
- Never `unwrap()`/`expect()` in prod without `expect("invariant: …")`.
- No `std::thread::sleep`/`std::fs` inside async fns; use
  `tokio::time::timeout` on all I/O; `spawn_blocking` for parsing work.
- Tests: `#[cfg(test)]` modules alongside code, no external services.
- Security: never log secrets; `secrecy::SecretString` for passwords.

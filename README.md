# DrugItems

```
██████╗ ██████╗ ██╗   ██╗ ██████╗ ██╗████████╗███████╗███╗   ███╗███████╗
██╔══██╗██╔══██╗██║   ██║██╔════╝ ██║╚══██╔══╝██╔════╝████╗ ████║██╔════╝
██║  ██║██████╔╝██║   ██║██║  ███╗██║   ██║   █████╗  ██╔████╔██║███████╗
██║  ██║██╔══██╗██║   ██║██║   ██║██║   ██║   ██╔══╝  ██║╚██╔╝██║╚════██║
██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║   ██║   ███████╗██║ ╚═╝ ██║███████║
╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚══════╝╚═╝     ╚═╝╚══════╝
```

---

## ◆ PULSE

The drug master is the hospital's shared dictionary — and someone may have
touched it. DrugItems answers from a spreadsheet export, the **source of
truth**: what the snapshot recorded, what the MySQL `drugitems` table holds
today, and exactly which cells drifted. Every column, every value. The app
reads the database with `SELECT`-only statements, never a write, and the
verdict arrives in Thai — the room it works in is a Thai hospital.

| ตรวจทุกคอลัมน์ ▣ | ต่าง/ใหม่/หาย ▣ | ค่าเก่า-ใหม่ ▣ | CSV export ▣ |
|---|---|---|---|

*v0.1.0 - the first integrity loop is sealed and serving.*

> Built with Tauri 2 + Leptos 0.8, judged by `drugitems-core`, read from
> MySQL by `drugitems-db`, trusted on `drugitems-snapshot` - never a write,
> never a plaintext secret.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One toolchain, two commands.

```
⟫ rustup target add wasm32-unknown-unknown   # via rust-toolchain.toml
⟫ cargo install trunk
⟫ cargo tauri dev
```

The release artifact: `⟫ cargo tauri build`

<details>
<summary>Prerequisites</summary>

- Rust **1.85+** (edition 2024) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev) - installed above
- Tauri 2 system dependencies for your platform
- A MySQL/MariaDB database (a read-only account is recommended)
- A spreadsheet export of the `drugitems` table (the snapshot, e.g.
  `drugiterms-20260908.xls` — 657 rows × 202 columns)

</details>

On first launch the Setup screen asks for the MySQL connection and the site
identity (ชื่อสถานบริการ) — the credentials are encrypted before they ever
touch disk.

---

## ◆ ANATOMY

Six crates, one boundary that never bends: the database is read, never
written.

- **Judges** — `drugitems-core` is the pure domain: the canonical compare
  engine (Excel date serials, BE/CE eras, numeric-looking text, NULL vs
  blank, CRLF vs LF) — testable without a database or a file.
- **Reads the truth** — `drugitems-snapshot` loads the exported spreadsheet
  via `calamine` into the source-of-truth table, strict about shape:
  duplicate columns or duplicate `icode`s are hard errors, never silent
  corruption.
- **Reads** — `drugitems-db` is the `sqlx` MySQL repository. Every
  statement is validated against an allow-list of `SELECT` / `SHOW` /
  `DESCRIBE` / `EXPLAIN` keywords before execution — read-only is enforced
  in code, not just recommended in a document.
- **Seals** — `drugitems-config` stores connection settings encrypted with
  `encryptman` (AES-256-GCM, HKDF-derived keys), the master key in the OS
  keychain; app settings stay in a plain, non-secret `settings.json`.
- **Speaks** — `drugitems-bridge` carries the verdict to the Leptos UI,
  which is Thai-only by design.
- **Serves** — `drugitems-app` is the Tauri shell: the commands, the diff
  report, and the CSV export.

---

## ◆ RITUALS

**The core ceremony** — a drug-master integrity check:

1. Open DrugItems, connect to MySQL. One configuration, remembered and
   sealed.
2. Choose the snapshot — the exported spreadsheet. Its columns, its values,
   its truth.
3. Compare. Read the verdict: ตรงกันทุกคอลัมน์ every value, or X items
   drifted.
4. Inspect: added and missing rows, the changed-column breakdown, and the
   per-column before-after table.
5. Export: the CSV carries every difference, ready to archive.

**The ceremony of the source of truth** — the spreadsheet is the expected
side in every diff. The app never "fixes" the database; it only reports.

**The ceremony of the read-only guard** — no `INSERT`, no `UPDATE`, no
`DELETE`, no DDL. The guard rejects anything that does not begin with
`SELECT` / `SHOW` / `DESCRIBE` / `EXPLAIN`.

---

## ◆ ECHOES

**Where this artifact is heading**

```
v0.1   ▸ scaffold, setup, snapshot load, full-table compare, CSV ▸ sealed
v0.2   ▸ remember the last snapshot, .csv snapshots (NULL-aware)
v0.3   ▸ A4 PDF report (Sarabun embedded, HarfBuzz shaped)
v0.4   ▸ multi-table / multi-snapshot comparisons
```

**Raising the artifact** — read `AGENTS.md` before touching a query: it
holds the data contract and the hard rules — read-only without exceptions,
no plaintext credentials, the canonical compare rules documented in
`drugitems-core`. The design language lives in `DESIGN.md`.

---

```
  ─────────────────────────────────────────
   Someone may have touched the drug master.
   The snapshot remembers. DrugItems tells.
  ─────────────────────────────────────────
```

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. Not affiliated with HOSxP, BMS, or any hospital system vendor.
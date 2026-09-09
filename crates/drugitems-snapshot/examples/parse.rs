//! Dev utility: parse a snapshot spreadsheet and print a summary.
//! Usage: `cargo run -p drugitems-snapshot --example parse -- <file.xls>`

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: parse <file.xls|.xlsx>");
        std::process::exit(2);
    }
    match drugitems_snapshot::load_snapshot(Path::new(&args[1])) {
        Ok(table) => {
            println!("file: {}", table.file_name.as_deref().unwrap_or("<none>"));
            println!(
                "columns ({}): {:?}",
                table.columns.len(),
                &table.columns[..table.columns.len().min(8)]
            );
            println!("rows: {}", table.rows.len());
            if let Some((code, cells)) = table.rows.iter().next() {
                println!("first row key: {code}");
                for column in table.columns.iter().take(8) {
                    println!("  {column}: {:?}", cells.get(column));
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}

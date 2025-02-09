use crate::squire;
use rusqlite::Connection;
use std::path::Path;

#[allow(dead_code)]
pub fn get_columns(manifest_db_path: &Path) -> rusqlite::Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(manifest_db_path)?;
    let mut col_smt = conn.prepare("PRAGMA table_info(Files)")?;
    let columns: Vec<String> = col_smt
        .query_map([], |row| {
            // The column name is in the second column (index 1)
            let col_name: String = row.get(1)?;
            Ok(col_name)
        })?
        .collect::<Result<Vec<String>, _>>()?;
    for col in columns {
        println!("{}", col);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn get_table(
    manifest_db_path: &Path,
    limit: Option<usize>,
) -> rusqlite::Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(manifest_db_path)?;
    let statement = match limit {
        Some(head) => format!(
            "SELECT * FROM Files {} LIMIT {}",
            squire::media_filter(),
            head
        ),
        None => format!("SELECT * FROM Files {}", squire::media_filter()),
    };
    let mut col_smt = conn.prepare(&statement)?;
    let columns: Vec<String> = col_smt
        .column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();
    let mut rows = col_smt.query([])?;
    println!("{:<20} {:<50}", "Column Name", "Value");
    while let Some(row) = rows.next()? {
        for (i, col_name) in columns.iter().enumerate() {
            let value: String = row.get::<_, String>(i).unwrap_or_default();
            println!("{:<20} {:<50}", col_name, value);
        }
        println!();
    }
    Ok(())
}

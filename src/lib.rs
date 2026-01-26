use anyhow::Result;
use std::fs;
use std::path::PathBuf;

mod constants;
use crate::constants::TOR_BRIDGE_PREFIX;


// обработать вариант когда нет файла или прав доступа
pub fn get_bridges_from_file(path: &PathBuf) -> Result<Vec<String>> {
    let contents = fs::read_to_string(&path)?;
    let lines = contents
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| s.starts_with(TOR_BRIDGE_PREFIX))
        .map(String::from)
        .collect();

    for line in &lines {
        println!("{}", line);
    }
    Ok(lines)
}
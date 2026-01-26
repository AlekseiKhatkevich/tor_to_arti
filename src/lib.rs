use anyhow::Result;
use chrono::{DateTime, Local};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
mod constants;


// обработать вариант когда нет файла или прав доступа
pub fn get_bridges_from_file(path: &PathBuf) -> Result<Vec<String>> {
    let contents = fs::read_to_string(&path)?;
    let lines = contents
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| s.starts_with(constants::TOR_BRIDGE_PREFIX))
        .map(String::from)
        .collect();

    Ok(lines)
}

pub fn print_bridges(bridges: Vec<String>) -> () {
    for bridge in &bridges {
        println!("{bridge}");
    }
}

pub fn print_last_modified(path: &Path) -> () {
    let mtime = fs::metadata(path).unwrap().modified().unwrap();
    let dt: DateTime<Local> = DateTime::from(mtime);
    println!("Tor bridges last modified: {} \n", dt.format("%Y-%m-%d %H:%M:%S"));
}
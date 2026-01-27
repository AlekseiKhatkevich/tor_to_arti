use anyhow::Result;
use chrono::{DateTime, Local};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
mod constants;
use toml_edit::{value, DocumentMut};
use tempfile::NamedTempFile;


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

pub fn print_last_modified(path: &Path) -> Result<()> {
    let mtime = fs::metadata(path)?.modified()?;
    let dt: DateTime<Local> = DateTime::from(mtime);
    println!("Tor bridges last modified: {} \n", dt.format("%Y-%m-%d %H:%M:%S"));
    Ok(())
}

pub fn save_bridges_in_arti_log(path: &Path, bridges: &[String]) -> Result<()> {
    let text = fs::read_to_string(path)?;
    let mut doc = text.parse::<DocumentMut>()?;

    doc["bridges"]["bridges"] = value(bridges.join("\n"));

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(doc.to_string().as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;

    Ok(())
}
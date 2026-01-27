use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use std::fs;
use std::io::Write;
use std::path::Path;
mod constants;
use tempfile::NamedTempFile;
use toml_edit::{value, DocumentMut};


pub fn get_bridges_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let contents = fs::read_to_string(&path)
        .map_err(
            |e| Error::msg(
                format!("failed to read {}: {}", path.as_ref().display(), e)
            )
        )?;

    let lines = contents
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| !s.starts_with("#"))
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

pub fn print_last_modified<P: AsRef<Path>>(path: P) -> Result<()> {
    let mtime = fs::metadata(path)?.modified()?;
    let dt: DateTime<Local> = DateTime::from(mtime);
    println!("Tor bridges last modified: {} \n", dt.format("%Y-%m-%d %H:%M:%S"));
    Ok(())
}

pub fn save_bridges_in_arti_log<P: AsRef<Path>>(path: P, bridges: Option<&[String]>) -> Result<()> {
    let path = path.as_ref();
    let text = fs::read_to_string(&path)?;
    let mut doc = text.parse::<DocumentMut>()?;

    if let Some(bridges) = bridges {
        doc["bridges"]["bridges"] = value(bridges.join("\n"));
    } else {
        doc["bridges"].as_table_mut().map(|t| t.remove("bridges"));
    }

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(doc.to_string().as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    fs::File::open(dir)?.sync_all()?;

    Ok(())
}
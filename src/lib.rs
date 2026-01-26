use std::path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use anyhow::Result;

pub fn get_bridges_from_file(path: &PathBuf) -> Vec<String> {
    let mut bridges_file = File::open(path).unwrap();
    let mut contents = String::new();
    bridges_file.read_to_string(&mut contents).unwrap();
    print!("{contents}");
    vec![]
}
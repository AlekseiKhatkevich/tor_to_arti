// /etc/tor/bridges.conf -> /home/hardcase/.config/arti
// cargo run --  --from-path /etc/tor/bridges.conf --to-path  /home/hardcase/.config/arti
use clap::Parser;
use std::path;

///Copy bridges from Tor file into Arti config file
#[derive(Parser)]
struct Cli {
    /// Path to Tor bridges file
    #[arg(short, long)]
    from_path: path::PathBuf,
    /// Path to Arti config file
    #[arg(short, long)]
    to_path: path::PathBuf,
}

fn main() {
    let cli = Cli::parse();

    println!("from: {:?}, into: {:?}", cli.from_path, cli.to_path);
}

// /etc/tor/bridges.conf -> /home/hardcase/.config/arti
// cargo run --  --from-path /etc/tor/bridges.conf --to-path  /home/hardcase/.config/arti
use clap::Parser;
use std::path;
use tor_to_arti::get_bridges_from_file;

///Copy bridges from Tor file into Arti config file
#[derive(Parser)]
struct Cli {
    /// Path to Tor bridges file
    #[arg(short = 'f', long)]
    from_path: path::PathBuf,
    /// Path to Arti config file
    #[arg(short = 't', long)]
    to_path: path::PathBuf,
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("from: {:?}, into: {:?}, dry_run: {:?}", cli.from_path, cli.to_path, cli.dry_run);
    
    get_bridges_from_file(&cli.from_path);
}

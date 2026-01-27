// /etc/tor/bridges.conf -> /home/hardcase/.config/arti
// Тесты не забыть
// Нормальную компиляцию сделать
// коммунты к функциям
use clap::Parser;
use clap::ValueHint;
use std::path::PathBuf;
use tor_to_arti::{
    get_bridges_from_file,
    print_bridges,
    print_last_modified,
    save_bridges_in_arti_log,
    reload_config,
};


///Copy bridges from Tor file into Arti config file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to Tor bridges file
    #[arg(short = 'f', long = "from", value_name = "FILE", value_hint = ValueHint::FilePath)]
    from: PathBuf,

    /// Path to Arti config file
    #[arg(short = 't', long = "to", value_name = "FILE", value_hint = ValueHint::FilePath)]
    to: PathBuf,

    /// Don't write changes; only show what would be done
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,

    /// Delete bridges section
    #[arg(long, action = clap::ArgAction::SetTrue)]
    delete_bridges: bool,

    /// Reload Arti config with SIGHUP
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    reload_config: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    print_last_modified(&cli.from).ok();
    let bridges = get_bridges_from_file(&cli.from)?;

    if cli.dry_run {
        print_bridges(&bridges);
    } else if cli.delete_bridges {
        save_bridges_in_arti_log(&cli.to, None)?;
    } else {
        save_bridges_in_arti_log(&cli.to, Some(&bridges))?;
    }
    if cli.reload_config {
        reload_config(None)?
    }

    Ok(())
}
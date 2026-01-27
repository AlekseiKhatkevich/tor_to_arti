// /etc/tor/bridges.conf -> /home/hardcase/.config/arti
// Тесты не забыть
// Нормальную компиляцию сделать
// печать даты модификации файла последней
// удалять секцию с мостами по флагу
//добавить сигнал перезагрузки арти
use clap::Parser;
use tor_to_arti::{get_bridges_from_file, print_bridges, print_last_modified, save_bridges_in_arti_log};
use std::path::PathBuf;
use clap::ValueHint;

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
}

fn main() {
    let cli = Cli::parse();
    let _ = print_last_modified(&cli.from);
    let bridges = get_bridges_from_file(&cli.from).unwrap();

    if cli.dry_run {
        print_bridges(bridges);
    } else if cli.delete_bridges{
        save_bridges_in_arti_log(&cli.to, &[String::new()]).unwrap();
    }
    else {
        save_bridges_in_arti_log(&cli.to, &bridges).unwrap();
    }
}

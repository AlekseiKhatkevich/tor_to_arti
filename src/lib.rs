use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs;
use std::io::Write;
use std::path::Path;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tempfile::NamedTempFile;
use toml_edit::{value, DocumentMut};

mod constants;

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

pub fn print_bridges(bridges: &[String]) -> () {
    for bridge in bridges {
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
        doc["bridges"]["bridges"] = value(format!("{}\n", bridges.join("\n")));
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

fn pids_by_name(name: &str) -> Vec<u32> {
    let mut sys = System::new_all();
    sys.refresh_specifics(
        RefreshKind::default().with_processes(ProcessRefreshKind::everything().without_tasks()),
    );

    sys.processes()
        .iter()
        .filter_map(|(&pid, proc_)| (proc_.name() == name).then(|| pid.as_u32()))
        .collect()
}


pub fn reload_config(name: Option<&str>) -> Result<(), anyhow::Error> {
    let name = name.unwrap_or(constants::ARTI_EXECUTABLE_NAME);
    let pids = pids_by_name(name);
    println!("Found {} PID(s) for '{}': {:?}", pids.len(), name, pids);

    for pid_u32 in pids {
        let pid = Pid::from_raw(i32::try_from(pid_u32)?);
        kill(pid, Signal::SIGHUP)?;
        }
    Ok(())
    }


#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};
    use std::thread::sleep as thread_sleep;
    use std::time::Duration;

    /// Запускаем unix sleep
    fn spawn_sleep(seconds: Duration) -> Result<Child> {
        let child = Command::new("sleep")
            .arg(seconds.as_secs().to_string())
            .spawn()?;

        Ok(child)
    }

    #[test]
    /// Позитивный тест ф-ции get_pids_by_name. Запускает sleep и получаем его пид.
    fn test_get_pids_by_name_positive() {
        let mut child = spawn_sleep(Duration::from_secs(5))
            .expect("Failed to spawn sleep command");
        let pid = child.id();
        assert!(pid >  0);

        thread_sleep(Duration::from_millis(100));
        let pids_from_f = pids_by_name("sleep");

        assert!(pids_from_f.contains(&pid), "spawned pid not found in result");
        child.kill().ok();
        child.wait().ok();
    }

    #[test]
    /// Негативный тест ф-ции get_pids_by_name. На несуществующее имя процесса
    /// ф-ция не отдает никаких pid-ов.
    fn test_get_pids_by_name_negative() {
        let proc_name = "!!!RaNdOm BuLLshit~~~";
        let pids_from_f = pids_by_name(proc_name);
        assert!(pids_from_f.is_empty());
    }

    #[test]
    /// Позитивный тест reload_config. Она должна отправлять SIGHUP процессу по его имени.
    /// Используем unix sleep который получив SIGHUP должен завершится. Если завершился -
    /// то значит SIGHUP получен.
    fn reload_config_positive() {
        let mut child = spawn_sleep(Duration::from_secs(5))
            .expect("Failed to spawn sleep command");
        let pid = child.id();
        assert!(pid >  0);

        thread_sleep(Duration::from_millis(100));
        reload_config(Some("sleep")).expect("reload_config failed");
        thread_sleep(Duration::from_millis(200));

        match child.try_wait().unwrap() {
            Some(status) => {
                assert!(!status.success());
            }
            None => {
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
                panic!("sleep still running after reload_config");
            }
        }
    }

    #[test]
    /// Позитивный тест ф-ции get_bridges_from_file. Даем ей путь на тестовый файл с тестовыми
    /// мостами и затем сравниваем результат с ожидаемым.
    fn test_get_bridges_from_file_positive() {
        let path = Path::new("src/tests/data/bridges.conf");
        let bridges = get_bridges_from_file(path).expect("Read bridges file");
        let expected = vec![
            "Bridge 64.65.62.199:443 4B0F565A6D8A005504EDF99CBC2DFE12E7D97D81".to_string(),
            "Bridge 37.187.74.97:9001 F745D5A34A289EF0C88544D0DC400B21120F5E81".to_string(),
            "Bridge 72.167.47.69:80 946D40F81F304814AE2D1A83CB4F219336E90ABF".to_string(),
        ];

        assert_eq!(bridges.len(), 3, "Vec should contain 3 elements");
        assert!(
            !bridges.contains(&String::from("UseBridges 1")),
            "Vec should not contains 'UseBridges'");
        assert_eq!(expected, bridges, "Expected bridges result mismatch");
    }
}

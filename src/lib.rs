use anyhow::{Context, Result};
use chrono;
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
        .with_context(|| format!("failed to read {}", path.as_ref().display()))?;

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

fn print_last_modified_to<W: Write, P: AsRef<Path>>(mut out: W, path: P) -> Result<()> {
    let mtime = fs::metadata(path)?.modified()?;
    let dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from(mtime);
    writeln!(out, "Tor bridges last modified: {} \n", dt.format("%Y-%m-%d %H:%M:%S"))?;
    Ok(())
}

pub fn print_last_modified<P: AsRef<Path>>(path: P) -> Result<()> {
    print_last_modified_to(std::io::stdout(), path)
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
    use chrono::{Local, TimeZone};
    use filetime::{set_file_times, FileTime};
    use std::io::Cursor;
    use std::process::{Child, Command};
    use std::thread::sleep as thread_sleep;
    use std::time::Duration;
    use std::time::SystemTime;

    /// Запускаем unix sleep
    fn spawn_sleep(seconds: Duration) -> Result<Child> {
        let child = Command::new("sleep")
            .arg(seconds.as_secs().to_string())
            .spawn()?;

        Ok(child)
    }

    /// Нормализуем строки в вектор
    fn normalize(s: &str) -> Vec<String> {
        s.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }


    #[test]
    /// Позитивный тест ф-ции get_pids_by_name. Запускает sleep и получаем его пид.
    fn test_get_pids_by_name_positive() {
        let mut child = spawn_sleep(Duration::from_secs(5))
            .expect("Failed to spawn sleep command");
        let pid = child.id();
        assert!(pid >  0, "wrong child pid");

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
        assert!(pids_from_f.is_empty(), "We strangely got some pids..why?");
    }

    #[test]
    /// Позитивный тест reload_config. Она должна отправлять SIGHUP процессу по его имени.
    /// Используем unix sleep который получив SIGHUP должен завершится. Если завершился -
    /// то значит SIGHUP получен.
    fn reload_config_positive() {
        let mut child = spawn_sleep(Duration::from_secs(5))
            .expect("Failed to spawn sleep command");
        let pid = child.id();
        assert!(pid >  0, "wrong child pid");

        thread_sleep(Duration::from_millis(100));
        reload_config(Some("sleep")).expect("reload_config failed");
        thread_sleep(Duration::from_millis(200));

        match child.try_wait().unwrap() {
            Some(status) => {
                assert!(!status.success(), "Child exit status should not be == 0");
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

    #[test]
    /// Негативный тест get_bridges_from_file. Если файл не существует по данному пути то
    /// получаем исключение и проверяем его.
    fn test_get_bridges_from_file_negative() {
        let path = Path::new("src/tests/data/random_bullshit.conf");
        let bridges = get_bridges_from_file(path);

        assert!(bridges.is_err(), "Expected error when reading nonexistent file");
        let err = bridges.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("failed to read"),
            "error message should contain 'failed to read', got: {}",
            msg
        );
        let io_err = err
            .downcast_ref::<std::io::Error>()
            .expect("cause should be std::io::Error");

        assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    /// Позитивный тест print_last_modified. Устанавливаем last-modified на файле и затем
    /// с помощью ф-ции печатаем его значение в консоль.
    fn test_print_last_modified_positive() {
        let path = Path::new("src/tests/data/bridges.conf");
        let desired = SystemTime::now();
        let ft = FileTime::from_system_time(desired);
        set_file_times(&path, ft, ft).unwrap();
        let mut buf = Cursor::new(Vec::new());

        print_last_modified_to(&mut buf, path).expect("print failed");

        let expected_prefix = "Tor bridges last modified: ";
        let output = String::from_utf8(buf.into_inner()).expect("invalid utf8");
        assert!(output.starts_with(expected_prefix));

        let date_part = output[expected_prefix.len() ..].trim();
        assert_eq!(date_part.len(), 19);

        let parsed = Local.datetime_from_str(date_part, "%Y-%m-%d %H:%M:%S").unwrap();
        let desired_dt: chrono::DateTime<Local> = chrono::DateTime::from(desired);
        let diff = (parsed.timestamp() - desired_dt.timestamp()).abs();
        assert!(diff <= 2, "mtime differs more than 2 seconds: {}", diff);
    }


    #[test]
    fn test_save_bridges_in_arti_log_positive() {
        let bridges_path = Path::new("src/tests/data/bridges.conf");
        let bridges = get_bridges_from_file(&bridges_path).unwrap();
        let config_path = Path::new("src/tests/data/config.toml");

        save_bridges_in_arti_log(&config_path, Some(&bridges)).ok();

        let text = fs::read_to_string(&config_path).unwrap();
        let doc = text.parse::<DocumentMut>().unwrap();

        let expected_bridges =
            "Bridge 64.65.62.199:443 4B0F565A6D8A005504EDF99CBC2DFE12E7D97D81
            Bridge 37.187.74.97:9001 F745D5A34A289EF0C88544D0DC400B21120F5E81
            Bridge 72.167.47.69:80 946D40F81F304814AE2D1A83CB4F219336E90ABF";

        let bridges_from_config_file =  doc["bridges"]["bridges"]
           .as_str()
           .expect("missing bridges.bridges");

        let exp_lines = normalize(expected_bridges).sort();
        let got_lines = normalize(bridges_from_config_file).sort();

        assert_eq!(exp_lines, got_lines);
    }

}

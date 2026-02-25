use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::error::Error;
use std::time::Duration;
use std::thread;

pub struct ProcessManager {
    logs_dir: PathBuf,
    pub pids_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(base_path: &PathBuf) -> Self {
        let pids_dir = base_path.join("data").join("pids");
        let logs_dir = base_path.join("logs");

        if !pids_dir.exists() { fs::create_dir_all(&pids_dir).unwrap(); }
        if !logs_dir.exists() { fs::create_dir_all(&logs_dir).unwrap(); }

        Self { logs_dir, pids_dir }
    }

    pub fn start(&self, name: &str, bin_path: &PathBuf, args: &[&str]) -> Result<u32, Box<dyn Error>> {
        
        if !self.logs_dir.exists() {
            std::fs::create_dir_all(&self.logs_dir)?;
        }
        
        let log_file_path = self.logs_dir.join(format!("{}.log", name));

        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_file_path)?;

        let log_file_clone = log_file.try_clone()?;

        let mut child = Command::new(bin_path)
            .args(args)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_file_clone))
            .spawn()?;

        thread::sleep(Duration::from_millis(100));
        
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "Servis anında çöktü (Çıkış kodu: {}). Lütfen 'cargo run -- logs {}' ile hatayı inceleyin.",
                status, name
            ).into());
        }

        let pid = child.id();

        if !self.pids_dir.exists() {
            std::fs::create_dir_all(&self.pids_dir)?;
        }
        
        let pid_file = self.pids_dir.join(format!("{}.pid", name));
        std::fs::write(&pid_file, pid.to_string())?;

        Ok(pid)
    }

    pub fn stop(&self, name: &str) -> Result<(), Box<dyn Error>> {
        let pid_file = self.pids_dir.join(format!("{}.pid", name));
        
        if !pid_file.exists() {
            return Err(format!("Service '{}' is not currently running.", name).into());
        }

        let pid_str = std::fs::read_to_string(&pid_file)?;
        let pid = pid_str.trim();

        #[cfg(unix)]
        let _ = Command::new("kill").arg("-9").arg(pid).output();

        #[cfg(windows)]
        let _ = Command::new("taskkill").arg("/F").arg("/PID").arg(pid).output();

        let _ = std::fs::remove_file(pid_file);

        Ok(())
    }
}
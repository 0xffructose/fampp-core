use std::fs::{self, File};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};
use std::error::Error;

pub struct ProcessManager {
    logs_dir: PathBuf,
    pids_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(base_path: &PathBuf) -> Self {
        let pids_dir = base_path.join("data").join("pids");
        let logs_dir = base_path.join("logs");

        // PID ve log klasörlerinin var olduğundan emin olalım
        if !pids_dir.exists() { fs::create_dir_all(&pids_dir).unwrap(); }
        if !logs_dir.exists() { fs::create_dir_all(&logs_dir).unwrap(); }

        Self { logs_dir, pids_dir }
    }

    /// Belirtilen çalıştırılabilir dosyayı (binary) arka planda başlatır
    pub fn start(&self, service_name: &str, bin_path: &PathBuf, args: &[&str]) -> Result<u32, Box<dyn Error>> {
        let pid_file = self.pids_dir.join(format!("{}.pid", service_name));

        // Eğer servis zaten çalışıyorsa engelle
        if pid_file.exists() {
            return Err(format!("{} zaten çalışıyor! Önce durdurmalısınız.", service_name).into());
        }

        // stdout ve stderr log dosyalarını oluştur
        let out_log = File::create(self.logs_dir.join(format!("{}_out.log", service_name)))?;
        let err_log = File::create(self.logs_dir.join(format!("{}_err.log", service_name)))?;

        // Komutu arka planda (spawn) çalıştır
        let child = Command::new(bin_path)
            .args(args)
            .stdout(Stdio::from(out_log))
            .stderr(Stdio::from(err_log))
            .spawn()?; // spawn() terminali kilitlemez, arka planda çalıştırır

        let pid = child.id();
        
        // PID değerini dosyaya yaz
        fs::write(&pid_file, pid.to_string())?;

        Ok(pid)
    }

    /// PID dosyasını okuyup çalışan servisi durdurur
    pub fn stop(&self, service_name: &str) -> Result<(), Box<dyn Error>> {
        let pid_file = self.pids_dir.join(format!("{}.pid", service_name));

        if !pid_file.exists() {
            return Err(format!("{} için çalışan bir süreç bulunamadı.", service_name).into());
        }

        let pid_str = fs::read_to_string(&pid_file)?;
        let pid_num: u32 = pid_str.trim().parse()?;

        // sysinfo ile işletim sisteminden (macOS/Windows/Linux) bağımsız olarak süreci bul ve öldür
        let sys = System::new_all();
        // sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from_u32(pid_num)) {
            process.kill();
            println!("🛑 {} (PID: {}) başarıyla durduruldu.", service_name, pid_num);
        } else {
            println!("⚠️ Süreç (PID: {}) zaten kapanmış veya bulunamadı.", pid_num);
        }

        // PID dosyasını temizle
        fs::remove_file(&pid_file)?;

        Ok(())
    }

    /// Sistemdeki servislerin anlık durumunu ekrana basar
    pub fn status(&self) {
        println!("{:<15} | {:<15} | {:<10}", "SERVİS", "DURUM", "PID");
        println!("{:-<45}", "");

        // İşletim sistemindeki tüm süreçlerin anlık bir kopyasını al
        let sys = System::new_all();

        // PID klasörünü oku
        if let Ok(entries) = fs::read_dir(&self.pids_dir) {
            let mut found_any = false;

            for entry in entries.flatten() {
                let path = entry.path();
                
                // Sadece .pid uzantılı dosyaları kontrol et
                if path.extension().and_then(|s| s.to_str()) == Some("pid") {
                    found_any = true;
                    let service_name = path.file_stem().unwrap().to_str().unwrap();
                    let pid_str = fs::read_to_string(&path).unwrap_or_default();
                    
                    if let Ok(pid_num) = pid_str.trim().parse::<u32>() {
                        // Süreç hala hayatta mı kontrolü
                        if sys.process(Pid::from_u32(pid_num)).is_some() {
                            println!("{:<15} | 🟢 ÇALIŞIYOR    | {:<10}", service_name, pid_num);
                        } else {
                            println!("{:<15} | 🔴 ÇÖKMÜŞ/KAPALI| {:<10}", service_name, "N/A");
                            // İsteğe bağlı: Kapalıysa çöpe dönen pid dosyasını otomatik sil
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }

            if !found_any {
                println!("⚠️ Kayıtlı hiçbir servis bulunamadı.");
            }
        } else {
            println!("⚠️ PID klasörü okunamadı veya henüz oluşturulmamış.");
        }
        println!("{:-<45}", "");
    }
}
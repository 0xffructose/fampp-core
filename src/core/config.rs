use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Sistemdeki kurulu ve çalışan paketlerin durumunu tutar
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppState {
    pub installed_packages: Vec<String>,
    pub running_services: Vec<String>,
}

pub struct ConfigManager {
    pub base_path: PathBuf,
}

impl ConfigManager {
    /// Yeni bir ConfigManager başlatır ve kök dizini belirler
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Kullanıcı ana dizini (Home) bulunamadı!");
        let base_path = home.join(".fampp");
        Self { base_path }
    }

    /// Gerekli klasör hiyerarşisini ve state.json dosyasını oluşturur
    pub fn init(&self) {
        // Oluşturulacak alt klasörler
        let dirs_to_create = ["packages", "www", "data", "logs"];
        
        for dir in dirs_to_create {
            let path = self.base_path.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path).expect("Klasör oluşturulamadı!");
                println!("📂 Oluşturuldu: {:?}", path);
            }
        }
        
        // state.json kontrolü ve oluşturulması
        let state_file = self.base_path.join("state.json");
        if !state_file.exists() {
            let default_state = AppState::default();
            let json = serde_json::to_string_pretty(&default_state)
                .expect("JSON dönüştürme hatası");
            
            fs::write(&state_file, json).expect("state.json yazılamadı!");
            println!("📄 Oluşturuldu: {:?}", state_file);
        }
    }

    /// Mevcut durumu state.json'dan okur
    #[allow(dead_code)]
    pub fn load_state(&self) -> AppState {
        let state_file = self.base_path.join("state.json");
        let data = fs::read_to_string(state_file).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or_default()
    }
}
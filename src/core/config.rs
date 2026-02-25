use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppState {
    pub installed_packages: Vec<String>,
    pub running_services: Vec<String>,
}

pub struct ConfigManager {
    pub base_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Kullanıcı ana dizini (Home) bulunamadı!");
        let base_path = home.join(".fampp");
        Self { base_path }
    }

    pub fn init(&self) {
        let dirs_to_create = ["packages", "www", "data", "logs"];
        
        for dir in dirs_to_create {
            let path = self.base_path.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path).expect("Klasör oluşturulamadı!");
                println!("📂 Oluşturuldu: {:?}", path);
            }
        }
        
        let state_file = self.base_path.join("state.json");
        if !state_file.exists() {
            let default_state = AppState::default();
            let json = serde_json::to_string_pretty(&default_state)
                .expect("JSON dönüştürme hatası");
            
            fs::write(&state_file, json).expect("state.json yazılamadı!");
            println!("📄 Oluşturuldu: {:?}", state_file);
        }
    }

    #[allow(dead_code)]
    pub fn load_state(&self) -> AppState {
        let state_file = self.base_path.join("state.json");
        let data = fs::read_to_string(state_file).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or_default()
    }
}
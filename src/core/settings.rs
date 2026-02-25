use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppSettings {
    pub language: String,
    pub ports: Ports,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ports {
    pub php: u16,
    pub mysql: u16,
}

impl AppSettings {
    pub fn load_or_create(base_path: &PathBuf) -> Self {
        let config_file = base_path.join("config.toml");
        
        if config_file.exists() {
            if let Ok(contents) = fs::read_to_string(&config_file) {
                if let Ok(settings) = toml::from_str(&contents) {
                    return settings;
                }
            }
        }
        
        let default_settings = AppSettings {
            language: "en".to_string(),
            ports: Ports {
                php: 8000,
                mysql: 3306,
            }
        };
        
        if let Ok(toml_string) = toml::to_string(&default_settings) {
            let _ = fs::write(config_file, toml_string);
        }
        
        default_settings
    }
}
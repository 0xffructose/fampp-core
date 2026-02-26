use std::collections::HashMap;
use std::path::Path;
use std::fs;
use colored::Colorize;

pub struct I18n {
    #[allow(dead_code)]
    lang: String,
    translations: HashMap<String, String>,
}

impl I18n {
    // Artık FAMPP'ın kök dizinini (base_path) de parametre olarak alıyoruz
    pub fn new(base_path: &Path, lang: &str) -> Self {
        let locales_dir = base_path.join("locales");

        // 1. Klasör yoksa oluştur
        if !locales_dir.exists() {
            let _ = fs::create_dir_all(&locales_dir);
        }

        // 2. Temel dil dosyalarını "Self-Healing" mantığıyla diske yaz (Eğer silinmişlerse/yoklarsa)
        // Böylece kullanıcı projeyi indirdiğinde manuel dosya taşımak zorunda kalmaz!
        let en_path = locales_dir.join("en.toml");
        if !en_path.exists() {
            let _ = fs::write(&en_path, include_str!("../../locales/en.toml"));
        }

        let tr_path = locales_dir.join("tr.toml");
        if !tr_path.exists() {
            let _ = fs::write(&tr_path, include_str!("../../locales/tr.toml"));
        }

        // 3. Kullanıcının config.toml'da istediği dili DİSKTEN CANLI OKU
        let target_file = locales_dir.join(format!("{}.toml", lang));
        let mut parsed: HashMap<String, String> = HashMap::new();

        if target_file.exists() {
            if let Ok(content) = fs::read_to_string(&target_file) {
                // unwrap_or_default() yerine MATCH kullanıp hatayı yakalıyoruz!
                match toml::from_str(&content) {
                    Ok(map) => parsed = map,
                    Err(e) => eprintln!("{} Çeviri dosyası ({}.toml) ayrıştırılamadı: {}", "⚠️".yellow(), lang, e),
                }
            }
        } else {
            // İstenen dil yoksa İngilizceye dön
            if let Ok(content) = fs::read_to_string(&en_path) {
                parsed = toml::from_str(&content).unwrap_or_default();
            }
        }

        Self {
            lang: lang.to_string(),
            translations: parsed,
        }
    }

    // String döndürüyoruz çünkü metinleri diskten canlı olarak (runtime) ürettik
    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
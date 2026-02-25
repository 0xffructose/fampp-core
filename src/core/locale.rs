pub struct I18n {
    lang: String,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        Self {
            lang: lang.to_lowercase(),
        }
    }

pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match self.lang.as_str() {
            "tr" => match key {
                "booting" => "Başlatılıyor...",
                "halting" => "Durduruluyor...",
                "success_start" => "arka planda başarıyla çalışıyor",
                "success_stop" => "temiz bir şekilde kapatıldı! (Zombi proses yok)",
                "specify_stop" => "Lütfen durdurulacak paketi belirtin (Örn: cargo run -- stop php)",
                "status_fetching" => "FAMPP ortam durumu getiriliyor...",
                "active" => "Aktif",
                "stopped" => "Durdu",
                "service" => "Servis",
                "status" => "Durum",
                "port_info" => "Port / Bilgi",
                "tip_monitor" => "Bir servisi izlemek için şunu kullanın:",
                "tip_boot" => "Ortamınızı başlatmak için şunu kullanın:",
                "no_active_services" => "Aktif bir servis yok",
                _ => key,
            },
            _ => match key { 
                "booting" => "Booting...",
                "halting" => "Halting...",
                "success_start" => "is running in the background",
                "success_stop" => "terminated cleanly! (No zombies left behind)",
                "specify_stop" => "Please specify a package name to stop (e.g., 'cargo run -- stop php').",
                "status_fetching" => "Fetching FAMPP environment status...",
                "active" => "Active",
                "stopped" => "Stopped",
                "service" => "Service",
                "status" => "Status",
                "port_info" => "Port / Info",
                "tip_monitor" => "To monitor a service use:",
                "tip_boot" => "To boot up your environment use:",
                "no_active_services" => "No active services",
                _ => key,
            }
        }
    }
}
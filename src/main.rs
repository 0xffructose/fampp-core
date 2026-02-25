mod core;

use std::path::PathBuf;
use std::path::Path;
use std::fs;
use crate::core::process::ProcessManager;
use crate::core::config::ConfigManager;
use crate::core::registry::get_package_info;
use crate::core::downloader;
use crate::core::extractor;

// Gerekli makroları içeri aktarıyoruz
use clap::{Parser, Subcommand};

/// Modüler ve Hafif AMP Stack Yöneticisi
#[derive(Parser)]
#[command(name = "fampp")]
#[command(about = "Gereksiz paketlerden arındırılmış, seç-indir mantıklı yerel geliştirme ortamı", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Yeni bir paket kurar (örn: php, nginx, mariadb)
    Install {
        /// Kurulacak paketin adı
        package: String,
        /// Belirli bir versiyon (Varsayılan: latest)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Kurulu servisleri başlatır
    Start {
        /// Tüm servisleri başlatır
        #[arg(short, long)]
        all: bool,
        /// Sadece belirli bir servisi başlatır
        package: Option<String>,
    },
    /// Çalışan servisleri durdurur
    Stop {
        /// Tüm servisleri durdurur
        #[arg(short, long)]
        all: bool,
        /// Sadece belirli bir servisi durdurur
        package: Option<String>,
    },
    /// Servislerin güncel çalışma durumunu gösterir
    Status,
    #[command(about = "Servisin anlık loglarını terminalde izler")]
    Logs {
        #[arg(help = "Paket adı (örn: php, mysql)")]
        package: String,
    },
}

/// İlgili dizin içinde hedef binary (exe) dosyasını alt klasörler dahil arar
fn find_executable(dir: &Path, bin_name: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(bin_name) {
                return Some(path);
            } else if path.is_dir() {
                if let Some(found) = find_executable(&path, bin_name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Sistemin temel klasörlerini ve ayarlarını başlat
    let config = ConfigManager::new();
    config.init();

    // &cli.command yerine cli.command kullanarak sahipliği (ownership) devralıyoruz.
    // Böylece bool değerleri pointer yerine doğrudan kullanabiliriz.
    match cli.command {
        Commands::Install { package, version } => {
            let v = version.as_deref().unwrap_or("latest");
            println!("🚀 Kurulum başlatılıyor: {} (Versiyon: {})", package, v);

            // Registry'den işletim sistemine uygun paket bilgilerini al
            match get_package_info(&package, v) {
                Some(info) => {
                    // URL'nin sonuna bakarak uzantıyı belirle (.zip mi, .tar.gz mi, .php mi?)
                    let ext = if info.url.ends_with(".tar.gz") || info.url.ends_with(".tgz") {
                        "tar.gz"
                    } else if info.url.ends_with(".php") {
                        "php"
                    } else {
                        "zip"
                    };
                    
                    // Artık sadece temp_archive_path değil, genel bir dosya yolu (temp_file_path)
                    let temp_file_path = config.base_path.join(format!("{}.{}", package, ext));
                    let package_dir = config.base_path.join("packages").join(&package);

                    // Gerçek URL ile indirme işlemini başlat
                    match downloader::download_file(&info.url, &temp_file_path).await {
                        Ok(_) => {
                            // --- YENİ EKLENEN KISIM: PAKET TÜRÜNE GÖRE İŞLEM ---
                            if package == "adminer" {
                                // Adminer tek bir dosyadır, arşive sokmadan doğrudan www içine kopyala
                                let www_dir = config.base_path.join("www");
                                if !www_dir.exists() {
                                    std::fs::create_dir_all(&www_dir).unwrap();
                                }
                                
                                let target_path = www_dir.join("adminer.php");
                                if let Err(e) = std::fs::copy(&temp_file_path, &target_path) {
                                    eprintln!("❌ Adminer kopyalanamadı: {}", e);
                                } else {
                                    // Başarılı kopyalamadan sonra ~/.m-amp klasöründeki gereksiz asıl dosyayı sil
                                    let _ = std::fs::remove_file(&temp_file_path);
                                    println!("✨ Adminer başarıyla 'www/adminer.php' olarak kuruldu!");
                                    println!("🌐 Arayüze erişmek için PHP'yi başlatıp http://127.0.0.1:8000/adminer.php adresine gidin.");
                                }
                            } else {
                                // Diğer tüm paketler (php, mysql vb.) için standart arşivden çıkarma işlemi
                                if let Err(e) = extractor::extract_archive(&temp_file_path, &package_dir) {
                                    eprintln!("❌ Çıkarma hatası: {}", e);
                                } else {
                                    println!("✨ {} başarıyla sisteme entegre edildi!", package);
                                }
                            }
                            // ---------------------------------------------------
                        }
                        Err(e) => eprintln!("❌ İndirme hatası: {}", e),
                    }
                }
                None => {
                    eprintln!("❌ Hata: '{}' paketi sisteminiz için desteklenmiyor veya bulunamadı.", package);
                }
            }
        }       
        Commands::Start { all: _, package } => {
            let pm = ProcessManager::new(&config.base_path);
            
            if let Some(pkg) = package {
                match get_package_info(&pkg, "latest") {
                    Some(info) => {
                        let package_dir = config.base_path.join("packages").join(&pkg);
                        
                        let bin_path = match find_executable(&package_dir, &info.bin_name) {
                            Some(path) => path,
                            None => {
                                eprintln!("❌ Hata: '{}' bulunamadı. Lütfen önce kurulumu yapın.", info.bin_name);
                                return;
                            }
                        };

                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(metadata) = std::fs::metadata(&bin_path) {
                                let mut perms = metadata.permissions();
                                perms.set_mode(0o755);
                                let _ = std::fs::set_permissions(&bin_path, perms);
                            }
                        }

                        let mut args: Vec<String> = Vec::new();
                        let mut actual_port = 8000; // Varsayılan PHP portumuz

                        // --- PAKETLERE ÖZEL BAŞLATMA ARGÜMANLARI ---
                        if pkg == "php" {
                            let www_dir = config.base_path.join("www");
                            
                            // Akıllı Port Tarayıcı: Boş bir port bulana kadar yukarı doğru tara
                            while std::net::TcpListener::bind(("127.0.0.1", actual_port)).is_err() {
                                actual_port += 1;
                            }

                            args = vec![
                                "-S".to_string(),
                                format!("127.0.0.1:{}", actual_port), // Bulunan boş portu ver
                                "-t".to_string(),
                                www_dir.to_str().unwrap().to_string(),
                            ];
                        } else if pkg == "mysql" {
                            let db_data_dir = config.base_path.join("data").join("mysql");
                            
                            // Gerçek kök dizini (basedir) buluyoruz
                            // Örn: .../packages/mysql/mysql-8.0.36/bin/mysqld -> .../packages/mysql/mysql-8.0.36/
                            let actual_basedir = bin_path.parent().unwrap().parent().unwrap();
                            
                            if !db_data_dir.exists() {
                                std::fs::create_dir_all(&db_data_dir).unwrap();
                            }

                            // Data klasörü boşsa MySQL'i ilklendir
                            let is_empty = std::fs::read_dir(&db_data_dir).unwrap().next().is_none();
                            if is_empty {
                                println!("⏳ MySQL ilk kez hazırlanıyor (Sistem tabloları oluşturuluyor)...");
                                let mut init_cmd = std::process::Command::new(&bin_path);
                                init_cmd.arg("--initialize-insecure") // Şifresiz root kullanıcısı
                                        .arg(format!("--basedir={}", actual_basedir.to_str().unwrap()))
                                        .arg(format!("--datadir={}", db_data_dir.to_str().unwrap()));
                                
                                let output = init_cmd.output().expect("❌ MySQL ilklendirilemedi!");
                                if !output.status.success() {
                                    eprintln!("❌ İlklendirme Hatası: {}", String::from_utf8_lossy(&output.stderr));
                                    return;
                                }
                                println!("✅ MySQL veritabanı dosyaları başarıyla oluşturuldu.");
                            }

                            // MySQL için logs klasörü oluştur ve log parametresini argümanlara ekle
                            let logs_dir = config.base_path.join("logs");
                            if !logs_dir.exists() { std::fs::create_dir_all(&logs_dir).unwrap(); }
                            
                            let log_file = logs_dir.join("mysql.log");

                            args = vec![
                                format!("--basedir={}", actual_basedir.to_str().unwrap()),
                                format!("--datadir={}", db_data_dir.to_str().unwrap()),
                                "--port=3306".to_string(),
                                format!("--log-error={}", log_file.to_str().unwrap()) // Hataları dosyaya yazdır!
                            ];
                        }

                        // --- ORTAK BAŞLATMA MANTIĞI ---
                        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                        
                        println!("🚀 {} servisi başlatılıyor...", pkg);
                        match pm.start(&pkg, &bin_path, &args_str) {
                            Ok(pid) => {
                                println!("✅ {} başarıyla başlatıldı! (PID: {})", pkg, pid);
                                if pkg == "php" {
                                    println!("🌐 Tarayıcıda açın: http://127.0.0.1:{}", actual_port);
                                    println!("💡 Adminer için: http://127.0.0.1:{}/adminer.php", actual_port);
                                } else if pkg == "mysql" {
                                    println!("🗄️  Bağlantı: 127.0.0.1:3306 | Kullanıcı: root | Şifre: (Yok)");
                                }
                            }
                            Err(e) => eprintln!("❌ {}", e),
                        }
                    }
                    None => eprintln!("❌ Hata: '{}' paketi desteklenmiyor.", pkg),
                }
            } else {
                println!("Lütfen bir paket adı belirtin (Örn: php veya mysql).");
            }
        }
        Commands::Stop { all, package } => {
            let pm = ProcessManager::new(&config.base_path);

            if let Some(pkg) = package {
                println!("🛑 {} servisi durduruluyor...", pkg);
                if let Err(e) = pm.stop(&pkg) {
                    eprintln!("❌ Durdurma hatası: {}", e);
                }
            } else if all {
                println!("⚠️ Tüm servisleri durdurma özelliği yakında eklenecek.");
            } else {
                println!("Hata: Lütfen bir paket adı belirtin veya --all bayrağını kullanın.");
            }
        }
        Commands::Status => {
            let pm = ProcessManager::new(&config.base_path);
            pm.status();
        }
        Commands::Logs { package } => {
            let log_file = config.base_path.join("logs").join(format!("{}.log", package));

            if !log_file.exists() {
                eprintln!("❌ Hata: '{}' için henüz bir log dosyası oluşmamış.", package);
                eprintln!("💡 İpucu: Önce servisi başlatıp biraz hata üretmesini bekleyin.");
                return;
            }

            println!("🔍 İzleniyor: {} (Çıkış yapmak için Ctrl+C tuşuna basın)", package);
            println!("--------------------------------------------------");

            // İşletim sisteminin kendi "tail -f" komutunu kullanarak anlık akışı terminale bağlıyoruz
            let mut tail_cmd = std::process::Command::new("tail");
            tail_cmd.arg("-f")
                    .arg(log_file.to_str().unwrap());

            // Bu komut, kullanıcı Ctrl+C yapana kadar terminali kilitler ve logları akıtır
            if let Err(e) = tail_cmd.status() {
                eprintln!("❌ Log izleyici başlatılamadı: {}", e);
            }
        }
    }
}
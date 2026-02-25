mod core;

use std::path::PathBuf;
use std::path::Path;
use std::fs;
use colored::Colorize;
use comfy_table::{Table, Cell, Color as TColor, Attribute};
use comfy_table::presets::UTF8_FULL;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use crate::core::process::ProcessManager;
use crate::core::config::ConfigManager;
use crate::core::settings::AppSettings;
use crate::core::locale::I18n;
use crate::core::registry::get_package_info;
use crate::core::downloader;
use crate::core::extractor;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fampp")]
#[command(about = "Gereksiz paketlerden arındırılmış, seç-indir mantıklı yerel geliştirme ortamı", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        package: String,
        #[arg(short, long)]
        version: Option<String>,
    },
    Start {
        #[arg(short, long)]
        all: bool,
        package: Option<String>,
    },
    Stop {
        #[arg(short, long)]
        all: bool,
        package: Option<String>,
    },
    Status,
    #[command(about = "Servisin anlık loglarını terminalde izler")]
    Logs {
        #[arg(help = "Paket adı (örn: php, mysql)")]
        package: String,
    },
}

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

    let config = ConfigManager::new();
    config.init();

    let app_settings = AppSettings::load_or_create(&config.base_path);
    let i18n = I18n::new(&app_settings.language);

    match cli.command {
        Commands::Install { package, version } => {
            let v = version.as_deref().unwrap_or("latest");
            println!("{} Fetching {} (v{}) from registry...", "📦".cyan(), package.bold().green(), v.yellow());

            match get_package_info(&package, v) {
                Some(info) => {
                    let ext = if info.url.ends_with(".tar.gz") || info.url.ends_with(".tgz") {
                        "tar.gz"
                    } else if info.url.ends_with(".php") {
                        "php"
                    } else {
                        "zip"
                    };
                    
                    let temp_file_path = config.base_path.join(format!("{}.{}", package, ext));
                    let package_dir = config.base_path.join("packages").join(&package);

                    match downloader::download_file(&info.url, &temp_file_path).await {
                        Ok(_) => {
                            if package == "adminer" {
                                let www_dir = config.base_path.join("www");
                                if !www_dir.exists() {
                                    std::fs::create_dir_all(&www_dir).unwrap();
                                }
                                let target_path = www_dir.join("adminer.php");

                                if let Err(e) = std::fs::copy(&temp_file_path, &target_path) {
                                    eprintln!("{} Failed to copy Adminer: {}", "❌".red(), e);
                                } else {
                                    let _ = std::fs::remove_file(&temp_file_path);
                                    println!("{} Adminer configured successfully!", "✨".green().bold());
                                }
                            } else {
                                if let Err(e) = extractor::extract_archive(&temp_file_path, &package_dir) {
                                    eprintln!("{} Extraction failed: {}", "❌".red(), e);
                                } else {
                                    println!("{} {} integrated successfully!", "✨".green().bold(), package.to_uppercase().green());
                                }
                            }
                        }
                        Err(e) => eprintln!("{} Download interrupted: {}", "❌".red(), e),
                    }
                }
                None => {
                    eprintln!("{} Package '{}' is not supported or not found in registry.", "⚠️".yellow(), package.bold());
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
                        let mut actual_port = app_settings.ports.php;

                        if pkg == "php" {
                            let www_dir = config.base_path.join("www");
                        
                            while std::net::TcpListener::bind(("127.0.0.1", actual_port)).is_err() {
                                actual_port += 1;
                            }

                            args = vec![
                                "-S".to_string(),
                                format!("127.0.0.1:{}", actual_port),
                                "-t".to_string(),
                                www_dir.to_str().unwrap().to_string(),
                            ];
                        } else if pkg == "mysql" {
                            let db_data_dir = config.base_path.join("data").join("mysql");
                            let actual_basedir = bin_path.parent().unwrap().parent().unwrap();
                            
                            if !db_data_dir.exists() {
                                std::fs::create_dir_all(&db_data_dir).unwrap();
                            }

                            let is_empty = std::fs::read_dir(&db_data_dir).unwrap().next().is_none();
                            if is_empty {
                                println!("⏳ MySQL ilk kez hazırlanıyor (Sistem tabloları oluşturuluyor)...");
                                let mut init_cmd = std::process::Command::new(&bin_path);
                                init_cmd.arg("--initialize-insecure")
                                        .arg(format!("--basedir={}", actual_basedir.to_str().unwrap()))
                                        .arg(format!("--datadir={}", db_data_dir.to_str().unwrap()));
                                
                                let output = init_cmd.output().expect("❌ MySQL ilklendirilemedi!");
                                if !output.status.success() {
                                    eprintln!("❌ İlklendirme Hatası: {}", String::from_utf8_lossy(&output.stderr));
                                    return;
                                }
                                println!("✅ MySQL veritabanı dosyaları başarıyla oluşturuldu.");
                            }

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

                        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                        
                        println!("{} Booting {} engine...", "⚡".cyan(), pkg.bold().green());
                        
                        match pm.start(&pkg, &bin_path, &args_str) {
                            Ok(pid) => {
                                println!("{} {} {} (PID: {})", 
                                    "✅".green(), 
                                    pkg.to_uppercase().bold(), 
                                    i18n.t("success_start"), 
                                    pid.to_string().yellow()
                                );
                                
                                if pkg == "php" {
                                    println!("   {} http://127.0.0.1:{}", "🌐 Localhost :".cyan().bold(), actual_port);
                                } else if pkg == "mysql" {
                                    println!("   {} 127.0.0.1:3306", "🗄️  Host :".blue().bold());
                                    println!("   {} root", "👤 User :".blue().bold());
                                    println!("   {} (None)", "🔑 Pass :".blue().bold());
                                }
                                println!();
                            }
                            Err(e) => {
                                eprintln!("{} Failed to start {}: {}", "❌".red().bold(), pkg.bold(), e.to_string().red());
                            }
                        }
                    }
                    None => eprintln!("❌ Hata: '{}' paketi desteklenmiyor.", pkg),
                }
            } else {
                println!("Lütfen bir paket adı belirtin (Örn: php veya mysql).");
            }
        }
        Commands::Stop { all: _, package } => {
            let pm = ProcessManager::new(&config.base_path);
            
            if let Some(pkg) = package {
                println!("{} {} {} engine...", "🛑".red(), i18n.t("halting"), pkg.bold().cyan());
                
                match pm.stop(&pkg) {
                    Ok(_) => {
                        println!("{} {} {}", 
                            "✅".green(), 
                            pkg.to_uppercase().bold(),
                            i18n.t("success_stop")
                        );
                    }
                    Err(e) => {
                        eprintln!("{} {}", "⚠️".yellow(), e.to_string().yellow());
                    }
                }
            } else {
                println!("{} {}", "⚠️".yellow(), i18n.t("specify_stop"));
            }
        }
        Commands::Status => {
            println!("{} {}", "🔍".cyan().bold(), i18n.t("status_fetching"));
            let pm = ProcessManager::new(&config.base_path);

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new(i18n.t("service")).add_attribute(Attribute::Bold).fg(TColor::Cyan),
                    Cell::new(i18n.t("status")).add_attribute(Attribute::Bold).fg(TColor::Cyan),
                    Cell::new("PID").add_attribute(Attribute::Bold).fg(TColor::Cyan),
                    Cell::new(i18n.t("port_info")).add_attribute(Attribute::Bold).fg(TColor::Cyan),
                ]);

            let services = vec![
                ("php", format!("127.0.0.1:{}+", app_settings.ports.php)), 
                ("mysql", format!("127.0.0.1:{}", app_settings.ports.mysql))
            ];

            let mut any_running = false;

            for (svc, info) in services {
                let pid_file = pm.pids_dir.join(format!("{}.pid", svc));
                
                let mut status_cell = Cell::new("Stopped").fg(TColor::Red);
                let mut pid_str = "-".to_string();

                if pid_file.exists() {
                    if let Ok(pid_content) = std::fs::read_to_string(&pid_file) {
                        status_cell = Cell::new("Active").fg(TColor::Green).add_attribute(Attribute::Bold);
                        pid_str = pid_content.trim().to_string();
                        any_running = true;
                    }
                }

                table.add_row(vec![
                    Cell::new(svc.to_uppercase()).add_attribute(Attribute::Bold),
                    status_cell,
                    Cell::new(pid_str).fg(TColor::Yellow),
                    Cell::new(info),
                ]);
            }

            println!("\n{table}\n");

            if any_running {
                println!("💡 {} {}", i18n.t("tip_monitor"), "'cargo run -- logs <service>'".yellow());
            } else {
                println!("💡 {} {}", i18n.t("tip_boot"), "'cargo run -- start <service>'".yellow());
            }
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

            let mut tail_cmd = std::process::Command::new("tail");
            tail_cmd.arg("-f")
                    .arg(log_file.to_str().unwrap());

            if let Err(e) = tail_cmd.status() {
                eprintln!("❌ Log izleyici başlatılamadı: {}", e);
            }
        }
    }
}
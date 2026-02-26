mod core;

use std::path::PathBuf;
use std::path::Path;
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use colored::Colorize;
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
#[command(disable_help_subcommand = true)]
#[command(disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
    Help,
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
    let i18n = I18n::new(&config.base_path, &app_settings.language);

    let active_command = cli.command.unwrap_or(Commands::Help);

    match active_command {
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
            print!("{} {}", "🔍".cyan().bold(), i18n.t("status_fetching"));
            io::stdout().flush().unwrap();
            
            let pm = ProcessManager::new(&config.base_path);
            
            let services = vec!["php", "mysql"];

            let mut any_running = false;
            let mut active_rows = Vec::new();

            for svc in services {
                let pid_file = pm.pids_dir.join(format!("{}.pid", svc));
                
                if pid_file.exists() {
                    if let Ok(pid_content) = std::fs::read_to_string(&pid_file) {

                        let port_file = pm.pids_dir.join(format!("{}.port", svc));
                        let actual_port = if let Ok(port_str) = std::fs::read_to_string(&port_file) {
                            port_str.trim().to_string() // Gerçek portu aldık!
                        } else {
                            // Eğer .port dosyası yoksa config.toml'daki varsayılana dön
                            if svc == "php" {
                                app_settings.ports.php.to_string()
                            } else {
                                app_settings.ports.mysql.to_string()
                            }
                        };

                        let info = format!("127.0.0.1:{}", actual_port);

                        any_running = true;
                        active_rows.push((
                            svc.to_uppercase(),
                            i18n.t("active"),
                            pid_content.trim().to_string(),
                            info,
                        ));
                    }
                }
            }

            thread::sleep(Duration::from_millis(150));

            print!("\r\x1b[2K");
            io::stdout().flush().unwrap();

            let v = "│".cyan(); 
            
            println!("{}", "┌──────────────┬──────────────┬─────────┬─────────────────────────┐".cyan());
            
            let h_svc = format!("{:<12}", i18n.t("service")).bold().cyan();
            let h_stat = format!("{:<12}", i18n.t("status")).bold().cyan();
            let h_pid = format!("{:<7}", "PID").bold().cyan();
            let h_port = format!("{:<23}", i18n.t("port_info")).bold().cyan();
            
            println!("{} {} {} {} {} {} {} {} {}", 
                v, h_svc, v, h_stat, v, h_pid, v, h_port, v
            );
            
            println!("{}", "├──────────────┼──────────────┼─────────┼─────────────────────────┤".cyan());

            if any_running {
                for (svc, status, pid, info) in active_rows {
                    let c_svc = format!("{:<12}", svc).bold();
                    let c_stat = format!("{:<12}", status).bold().green();
                    let c_pid = format!("{:<7}", pid).yellow();
                    let c_port = format!("{:<23}", info);
                    
                    println!("{} {} {} {} {} {} {} {} {}", 
                        v, c_svc, v, c_stat, v, c_pid, v, c_port, v
                    );
                }
            } else {
                let msg = i18n.t("no_active_services");
                let empty_msg = format!("{:^65}", msg).bold().red();
                println!("{}{}{}", v, empty_msg, v); 
            }

            println!("{}\n", "└──────────────┴──────────────┴─────────┴─────────────────────────┘".cyan());

            if any_running {
                println!("💡 {} {}", i18n.t("tip_monitor"), "'fampp logs <service>'".yellow());
            } else {
                println!("💡 {} {}", i18n.t("tip_boot"), "'fampp start <service>'".yellow());
            }
        }
        Commands::Logs { package } => {
            let pkg = package.to_lowercase();
            
            let log_file = config.base_path.join("logs").join(format!("{}.log", pkg));

            if !log_file.exists() {
                eprintln!("{} {} {}", "⚠️".yellow(), i18n.t("log_not_found"), pkg.bold());
                return;
            }

            // --- YENİ ŞIK LOG DASHBOARD TASARIMI ---
            println!("\n{}", "════════════════════════════════════════════════════════════════".cyan());
            println!(" {} {} {}", 
                "📡".green(), 
                i18n.t("log_live_stream").bold(),
                pkg.to_uppercase().bold().yellow()
            );
            println!(" {} {}", 
                "🛑".red(), 
                i18n.t("log_exit_tip").dimmed()
            );
            println!("{}\n", "════════════════════════════════════════════════════════════════".cyan());

            use std::io::{Read, Seek, SeekFrom};
            use std::fs::File;
            use std::thread;
            use std::time::Duration;

            if let Ok(mut file) = File::open(&log_file) {
                let mut buffer = String::new();
                
                // Sadece yeni logları ekrana basmak için dosyanın sonuna gidiyoruz
                let mut pos = file.seek(SeekFrom::End(0)).unwrap_or(0);

                loop {
                    file.seek(SeekFrom::Start(pos)).unwrap();
                    buffer.clear();
                    
                    if let Ok(bytes_read) = file.read_to_string(&mut buffer) {
                        if bytes_read > 0 {
                            // Logları göz yormaması için "bright_black" (Koyu Gri/Soluk) renkte basıyoruz
                            print!("{}", buffer.bright_black());
                            io::stdout().flush().unwrap();
                            
                            pos += bytes_read as u64;
                        }
                    }
                    
                    // Akıcı bir okuma için minik gecikme
                    thread::sleep(Duration::from_millis(300));
                }
            } else {
                eprintln!("{} {}", "❌".red(), i18n.t("log_read_error"));
            }
        }
        Commands::Help => {
            let ascii_logo = r#"
 ________  ______   __       __  _______   _______  
/        |/      \ /  \     /  |/       \ /       \ 
$$$$$$$$//$$$$$$  |$$  \   /$$ |$$$$$$$  |$$$$$$$  |
$$ |__   $$ |__$$ |$$$  \ /$$$ |$$ |__$$ |$$ |__$$ |
$$    |  $$    $$ |$$$$  /$$$$ |$$    $$/ $$    $$/ 
$$$$$/   $$$$$$$$ |$$ $$ $$/$$ |$$$$$$$/  $$$$$$$/  
$$ |     $$ |  $$ |$$ |$$$/ $$ |$$ |      $$ |      
$$ |     $$ |  $$ |$$ | $/  $$ |$$ |      $$ |      
$$/      $$/   $$/ $$/      $$/ $$/       $$/       
     
            "#;

            println!("{}", ascii_logo.red().bold());
            
            println!("{} {} {}", 
                i18n.t("help_usage").yellow().bold(), 
                "fampp".white().bold(), // BURASI DEĞİŞTİ
                "<command> [args]".green()
            );
            
            println!("\n{}\n", i18n.t("help_commands").yellow().bold());

            let commands = vec![
                ("install <pkg>", i18n.t("cmd_install")),
                ("start <pkg>", i18n.t("cmd_start")),
                ("stop <pkg>", i18n.t("cmd_stop")),
                ("status", i18n.t("cmd_status")),
                ("logs <pkg>", i18n.t("cmd_logs")),
                ("help", i18n.t("cmd_help")),
            ];

            for (cmd, desc) in commands {
                println!("  {:<15} {}", cmd.green().bold(), desc);
            }
            
            println!(); 
        }
    }
}
use std::env;

/// Bir paketin işletim sistemine özel indirme ve çalıştırma bilgileri
pub struct PackageInfo {
    #[allow(dead_code)]
    pub name: String,
    pub url: String,
    pub bin_name: String, // Windows için "php.exe", macOS için "php" vb.
}

/// İstenen paketin sistem uyumlu bilgilerini döndürür
pub fn get_package_info(package_name: &str, _version: &str) -> Option<PackageInfo> {
    let os = env::consts::OS;       // Örn: "macos", "windows", "linux"
    let arch = env::consts::ARCH;   // Örn: "x86_64", "aarch64"

    match package_name.to_lowercase().as_str() {
        "php" => {
            let (url, bin_name) = match os {
                "windows" => (
                    "https://windows.php.net/downloads/releases/php-8.2.12-nts-Win32-vs16-x64.zip",
                    "php.exe"
                ),
                "macos" => {
                    let mac_url = if arch == "aarch64" {
                        "https://dl.static-php.dev/static-php-cli/common/php-8.2.12-cli-macos-aarch64.tar.gz" // Apple Silicon
                    } else {
                        "https://dl.static-php.dev/static-php-cli/common/php-8.2.12-cli-macos-x86_64.tar.gz" // Intel Mac
                    };
                    (mac_url, "php")
                },               
                _ => return None, // Linux veya desteklenmeyen OS
            };

            Some(PackageInfo {
                name: package_name.to_string(),
                url: url.to_string(),
                bin_name: bin_name.to_string(),
            })
        },
        "mysql" => {
            let (url, bin_name) = match os {
                "windows" => (
                    "https://github.com/0xffructose/fampp-core/releases/download/BinaryUpdate/mysql-8.4.8-winx64.zip",
                    "mysqld.exe"
                ),
                "macos" => {
                    let mac_url = if arch == "aarch64" {
                        "https://github.com/0xffructose/fampp-core/releases/download/BinaryUpdate/mysql-8.4.8-macos15-arm64.tar.gz"
                    } else {
                        "https://github.com/0xffructose/fampp-core/releases/download/BinaryUpdate/mysql-8.4.8-macos15-x86_64.tar.gz"
                    };
                    (mac_url, "mysqld")
                },
                _ => return None,
            };

            Some(PackageInfo {
                name: package_name.to_string(),
                url: url.to_string(),
                bin_name: bin_name.to_string(),
            })
        },
        "adminer" => {
            Some(PackageInfo {
                name: package_name.to_string(),
                // Adminer'ın resmi ve doğrudan indirme linki (Tek bir PHP dosyası)
                url: "https://github.com/vrana/adminer/releases/download/v4.8.1/adminer-4.8.1.php".to_string(),
                bin_name: "adminer.php".to_string(),
            })
        },
        _ => None,
    }
}
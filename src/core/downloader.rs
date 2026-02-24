use std::path::PathBuf;
use std::process::Command;
use std::error::Error;

pub async fn download_file(url: &str, dest_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("🌐 İndiriliyor (Native cURL): {}", url);
    
    let mut cmd = Command::new("curl");
    
    cmd.arg("-f") // Hatalarda sessizce başarısız ol (bozuk dosya kaydetme)
       .arg("-L") // GitHub'ın yönlendirmelerini (Redirect) takip et
       .arg("-#") // Terminalde şık bir ilerleme çubuğu göster
       .arg("-o") // Çıktıyı hedef dosyaya yaz
       .arg(dest_path.to_str().unwrap())
       .arg(url);

    let status = cmd.status()?;

    if status.success() {
        println!("✅ İndirme tamamlandı: {:?}", dest_path);
        Ok(())
    } else {
        if dest_path.exists() {
            let _ = std::fs::remove_file(dest_path);
        }
        Err(format!("İndirme başarısız oldu. curl çıkış kodu: {}", status.code().unwrap_or(1)).into())
    }
}
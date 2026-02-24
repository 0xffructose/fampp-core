use std::fs;
use std::path::PathBuf;
use std::error::Error;
use zip::read::ZipArchive;
use flate2::read::GzDecoder;
use tar::Archive;

/// Arşivin uzantısına göre otomatik olarak ZIP veya TAR.GZ çıkarır
pub fn extract_archive(archive_path: &PathBuf, extract_to: &PathBuf) -> Result<(), Box<dyn Error>> {
    let path_str = archive_path.to_string_lossy().to_lowercase();

    // Hedef klasör yoksa oluştur
    if !extract_to.exists() {
        fs::create_dir_all(extract_to)?;
    }

    // Uzantıya göre yönlendirme yap
    if path_str.ends_with(".zip") {
        extract_zip(archive_path, extract_to)?;
    } else if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        extract_tar_gz(archive_path, extract_to)?;
    } else {
        return Err("Desteklenmeyen arşiv formatı!".into());
    }

    println!("🎉 Çıkarma işlemi başarılı: {:?}", extract_to);
    
    // Temizlik: İşi biten arşiv dosyasını sil
    fs::remove_file(archive_path)?;
    
    Ok(())
}

fn extract_zip(zip_path: &PathBuf, extract_to: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("📦 ZIP arşivi çıkartılıyor: {:?}", zip_path);
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(extract_to)?;
    Ok(())
}

fn extract_tar_gz(tar_path: &PathBuf, extract_to: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("📦 TAR.GZ arşivi çıkartılıyor: {:?}", tar_path);
    let file = fs::File::open(tar_path)?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    archive.unpack(extract_to)?;
    Ok(())
}
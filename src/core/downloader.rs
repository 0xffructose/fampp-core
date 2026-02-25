use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::error::Error;
use colored::Colorize;

pub async fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let res = client.get(url).send().await?;
    let total_size = res.content_length().ok_or("Failed to get content length from server")?;

    let pb = ProgressBar::new(total_size);
    
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {msg}\n{elapsed_precise} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})"
        )
        .unwrap()
        .progress_chars("█▓▒░ ")
    );
    
    pb.set_message(format!("{} Veri akışı sağlanıyor...", "🌐".cyan()));

    let mut file = File::create(dest).await?;
    let mut downloaded: u64 = 0;
    
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_and_clear();

    Ok(())
}
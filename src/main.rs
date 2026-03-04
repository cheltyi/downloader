use dunce;
use futures_util::StreamExt;
use reqwest;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory: &str = "downloads";
    let filename: &str = "image.gif";
    let path = Path::new(directory).join(filename);
    let args: Vec<String> = env::args().collect();
    let domain: &str = "https://static.klipy.com/ii/4e7bea9f7a3371424e6c16ebc93252fe/84/ef/1ocFw0eIBjDcaP.gif";
    fs::create_dir_all(directory).await?;
    if args.len() > 1 && args[1] == "--list" {
        list(directory).await?;
        return Ok(());
    }
    else if args.len() > 1 {
        println!("usage: {} [--list]", args[0]);
    } else {
        println!("using domain: \"{}\", connecting...", domain);
        download_file(&domain, &path).await?;
        let absolute_path = dunce::canonicalize(&path)?;
        println!(
            "{} downloaded to {}! its absolute path is {}",
            domain,
            filename,
            absolute_path.display()
        );
    }
    Ok(())
}

async fn download_file(dest: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    println!("client created");
    let res = client.get(dest).send().await?.error_for_status()?;
    println!("connected, collecting total size...");
    let total_size = res.content_length().ok_or("failed to get content length")?;
    let (size, unit) = if total_size < 1024 {
        (total_size as f64, "bytes")
    } else if total_size < 1024 * 1024 {
        (total_size as f64 / 1024.0, "KiB")
    } else if total_size < 1024 * 1024 * 1024 {
        (total_size as f64 / 1024.0 / 1024.0, "MiB")
    } else {
        (total_size as f64 / 1024.0 / 1024.0 / 1024.0, "GiB")
    };
    println!("total size: {:.2} {}", size, unit);

    let mut file = File::create(path).await?;
    println!("file created");
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
    }
    Ok(())
}

async fn list(directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(directory).await?;
    while let Some(entry) = dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            entries.push(name);
        }
    }
    println!("{:#?}", entries);
    Ok(())
}

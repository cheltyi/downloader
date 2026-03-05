use clap::Parser;
use dunce;
use futures_util::StreamExt;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use reqwest;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Shows files in directory and closes program.
    #[arg(short, long)]
    content: bool,

    /// Sets custom domain.
    #[arg(
        short,
        long,
        default_value = "https://static.klipy.com/ii/4e7bea9f7a3371424e6c16ebc93252fe/84/ef/1ocFw0eIBjDcaP.gif"
    )]
    link: String,

    /// Sets custom output file.
    #[arg(short, long, default_value = "image.gif")]
    filename: String,

    /// Sets custom output directory.
    #[arg(short, long, default_value = "downloads")]
    directory: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let directory: &str = args.directory.as_str();
    let filename: &str = args.filename.as_str();
    let content: bool = args.content;
    if content {
        contentf(Path::new(directory)).await?;
        return Ok(());
    }

    fs::create_dir_all(directory).await?;

    let path = Path::new(directory).join(filename);
    let link: &str = args.link.as_str();


    println!("using link: \"{}\", connecting...", link);
    download_file(&link, &path).await?;
    let absolute_path = dunce::canonicalize(&path)?;
    println!(
        "{} downloaded to {}! its absolute path is {}", link, filename, absolute_path.display()
    );

    Ok(())
}

async fn download_file(dest: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    println!("client created");
    let res = client.get(dest).send().await?.error_for_status()?;
    println!("connected, collecting total size...");
    let total_size = res.content_length().ok_or("failed to get content length")?;
    println!("total size: {}", HumanBytes(total_size));

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    pb.enable_steady_tick(Duration::from_millis(75));
    pb.set_message(format!("downloading to {}", path.display()));

    let mut file = File::create(&path).await?;
    pb.println("file created");
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_with_message("download complete");
    Ok(())
}

async fn contentf(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !directory.exists() {
        println!("directory '{}' does not exist", directory.display());
        return Ok(());
    }
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(directory).await?;
    while let Some(entry) = dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            entries.push(name);
        }
    }
    if entries.is_empty() {
        println!("directory '{}' is empty", directory.display());
        return Ok(());
    }
    println!("contents of {}:", directory.display());
    for name in entries {
        println!("  - {}", name);
    }
    Ok(())
}

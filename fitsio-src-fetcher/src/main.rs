use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use tempfile::NamedTempFile;

#[derive(Parser)]
struct Args {
    /// Version of cfitsio to fetch
    version: String,

    /// Location to unpack the source to
    #[clap(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let url = download_url(&args.version);

    let archive_file = download_archive(&url).context("downloading source archive")?;
    unpack_archive_to(archive_file.path(), &args.output).context("unpacking archive")?;

    Ok(())
}

fn download_url(version: &str) -> String {
    format!(
        "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-{}.tar.gz",
        version
    )
}

fn download_archive(url: &str) -> anyhow::Result<NamedTempFile> {
    eprintln!("downloading from '{url}'");
    let mut response = reqwest::blocking::get(url).context("failed to send request")?;
    response
        .error_for_status_ref()
        .context("bad status code from download url")?;
    let mut output_file = NamedTempFile::new().context("creating temporary output file path")?;
    eprintln!(
        "saving archive to temporary path: '{}'",
        output_file.path().display()
    );
    std::io::copy(&mut response, &mut output_file).context("copying file content")?;
    Ok(output_file)
}

fn unpack_archive_to(archive_path: &Path, destination_path: &Path) -> anyhow::Result<()> {
    eprintln!("unpacking archive into '{}'", destination_path.display());
    std::fs::create_dir_all(destination_path).context("creating output directory")?;
    let result = std::process::Command::new("tar")
        .args([
            "-C",
            &format!("{}", destination_path.display()),
            "-xf",
            &format!("{}", archive_path.display()),
            "--strip-components",
            "1",
            "--exclude",
            "docs",
        ])
        .spawn()
        .context("creating tar process")?
        .wait()
        .context("waiting for child process")?;
    anyhow::ensure!(result.success(), "failed to unpack archive");
    Ok(())
}

#[macro_use]
extern crate futures;

use anyhow::Context;
use async_std::{fs::File as AsyncFile, io as async_io};
use exit_status_ext::ExitStatusExt;

use std::{fs::File, path::Path, process::Stdio};

// NOTE: switch to async-std when it supports Command.
use tokio::net::process::Command;

pub async fn generate_logs() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir().context("failed to fetch temporary directory")?;
    let temppath = tempdir.path();

    let xorg_log = &temppath.join("Xorg.0.log");
    let syslog = &temppath.join("syslog");

    try_join!(
        dmidecode(tempfile(temppath, "dmidecode")?),
        lspci(tempfile(temppath, "lspci")?),
        lsusb(tempfile(temppath, "lsusb")?),
        dmesg(tempfile(temppath, "dmesg")?),
        journalctl(tempfile(temppath, "journalctl")?),
        upower(tempfile(temppath, "upower")?),
        copy(Path::new("/var/log/Xorg.0.log"), xorg_log),
        copy(Path::new("/var/log/syslog"), syslog),
    )?;

    Command::new("tar")
        .arg("-C")
        .arg(temppath)
        .arg("-Jpcf")
        .arg("system76-logs.tar.xz")
        .args(&[
            "dmidecode",
            "lspci",
            "lsusb",
            "dmesg",
            "journalctl",
            "upower",
            "Xorg.0.log",
            "syslog",
        ])
        .status()
        .await
        .context("tar failed to spawn")?
        .as_result()
        .context("tar exited in failure")
}

async fn command(command: &str, args: &[&str], output: File) -> anyhow::Result<()> {
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output)
        .status()
        .await
        .with_context(|| format!("{} failed to spawn", command))?
        .as_result()
        .with_context(|| format!("{} exited in failure", command))
        .map(|_| ())
}

async fn copy(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let source = async move { AsyncFile::open(source).await.context("failed to open source") };

    let dest = async move { AsyncFile::create(dest).await.context("failed to create dest") };

    let (mut source, mut dest) = try_join!(source, dest)?;
    async_io::copy(&mut source, &mut dest).await.context("failed to copy").map(|_| ())
}

async fn dmesg(file: File) -> anyhow::Result<()> { command("dmesg", &[], file).await }

async fn dmidecode(file: File) -> anyhow::Result<()> { command("dmidecode", &[], file).await }

async fn journalctl(file: File) -> anyhow::Result<()> {
    command("journalctl", &["--since", "yesterday"], file).await
}

async fn lspci(file: File) -> anyhow::Result<()> { command("lspci", &["-vv"], file).await }

async fn lsusb(file: File) -> anyhow::Result<()> { command("lsusb", &["-vv"], file).await }

async fn upower(file: File) -> anyhow::Result<()> { command("upower", &["-d"], file).await }

fn tempfile(path: &Path, command: &str) -> anyhow::Result<File> {
    File::create(path.join(command))
        .with_context(|| format!("failed to create temporary file for {}", command))
}

use anyhow::Context;
use as_result::IntoResult;
use async_std::{fs::File as AsyncFile, io as async_io};
use pidfd::PidFd;
use std::{
    fs::File,
    path::Path,
    process::{Command, Stdio},
};

pub async fn generate() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir().context("failed to fetch temporary directory")?;
    let temppath = tempdir.path();

    let apt_history = &temppath.join("apt_history");
    let xorg_log = &temppath.join("Xorg.0.log");
    let syslog = &temppath.join("syslog");

    try_join!(
        dmidecode(tempfile(temppath, "dmidecode")?),
        lspci(tempfile(temppath, "lspci")?),
        lsusb(tempfile(temppath, "lsusb")?),
        dmesg(tempfile(temppath, "dmesg")?),
        journalctl(tempfile(temppath, "journalctl")?),
        upower(tempfile(temppath, "upower")?),
        copy(Path::new("/var/log/apt/history.log"), apt_history),
        copy(Path::new("/var/log/Xorg.0.log"), xorg_log),
        copy(Path::new("/var/log/syslog"), syslog),
    )?;

    Command::new("tar")
        .arg("-C")
        .arg(temppath)
        .arg("-Jpcf")
        .arg("system76-logs.tar.xz")
        .args(&[
            "apt_history",
            "dmesg",
            "dmidecode",
            "journalctl",
            "lspci",
            "lsusb",
            "syslog",
            "upower",
            "Xorg.0.log",
        ])
        .spawn()
        .map(|child| PidFd::from(&child))
        .context("tar failed to spawn")?
        .into_future()
        .await
        .and_then(IntoResult::into_result)
        .context("tar exited in failure")
}

async fn command(command: &str, args: &[&str], output: File) -> anyhow::Result<()> {
    info!("fetching output from `{} {}`", command, args.join(" "));
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output)
        .spawn()
        .map(|child| PidFd::from(&child))
        .with_context(|| format!("{} failed to spawn", command))?
        .into_future()
        .await
        .and_then(IntoResult::into_result)
        .with_context(|| format!("{} exited in failure", command))
}

async fn copy(source: &Path, dest: &Path) -> anyhow::Result<()> {
    info!("copying logs from {}", source.display());
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

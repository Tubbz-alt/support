use super::apt;

use anyhow::Context;
use as_result::IntoResult;
use async_std::{fs::File, prelude::*};

use deb_control::prelude::*;
use deb_diversion::*;
use futures_codec::FramedRead;
use pidfd::PidFd;

use std::{ffi::OsStr, os::unix::ffi::OsStrExt, process::Command, str};

const STATUS: &str = "/var/lib/dpkg/status";

/// Applies the repairs for this subcommand
pub async fn repair() -> anyhow::Result<()> {
    let reinstall_packages = &mut Vec::new();

    info!("removing NVIDIA package diverts");
    fix_bad_diverts().await?;

    info!("purging NVIDIA packages");
    purge(reinstall_packages).await?;

    apt::repair().await?;

    info!("reinstalling NVIDIA packages");
    reinstall(reinstall_packages).await
}

/// Removes any files that have a divert from a previously-installed NVIDIA package
async fn fix_bad_diverts() -> anyhow::Result<()> {
    let file = File::open(DIVERSIONS).await.context("failed to open dpkg status file")?;

    let mut frames = FramedRead::new(file, DiversionDecoder::default());

    while let Some(event) = frames.next().await {
        let event = event.context("failed to decode dpkg diversion entry")?;

        if twoway::find_bytes(&event.by, b"nvidia").is_some() {
            Command::new("dpkg-divert")
                .arg("--remove")
                .arg(OsStr::from_bytes(&event.to))
                .spawn()
                .map(PidFd::from)
                .context("dpkg-divert failed to spawn")?
                .into_future()
                .await
                .and_then(IntoResult::into_result)
                .context("dpkg-divert exited in failure")?;
        }
    }

    Ok(())
}

/// Finds all packages pertaining to NVIDIA which are installed
///
/// Additionally records any packages which were installed, which should be reinstalled.
async fn packages_matching(
    packages: &mut Vec<String>,
    reinstall: &mut Vec<String>,
    name: &str,
) -> anyhow::Result<()> {
    let file = File::open(STATUS).await.context("failed to open dpkg status file")?;

    let mut frames = FramedRead::new(file, ControlDecoder::default());

    while let Some(event) = frames.next().await {
        let event = event.context("failed to decode dpkg control entry")?;
        let event = str::from_utf8(&event).expect("not UTF8");

        let mut control = Control::new(&event);

        let package = control.next().context("dpkg control entry did not contain any fields")?;

        if package.value.contains(name) {
            control.filter(|entry| entry.key == "Architecture").next().map(|entry| {
                entry.value.split_ascii_whitespace().for_each(|arch| {
                    if reinstall_package(&package.value) {
                        reinstall.push(package.value.into());
                    }
                    packages.push(if arch == "all" {
                        package.value.into()
                    } else {
                        [package.value, ":", arch].concat().into()
                    });
                });
            });
        }
    }

    Ok(())
}

/// Purges all NVIDIA-related packages from the system
async fn purge(reinstall: &mut Vec<String>) -> anyhow::Result<()> {
    let packages = &mut Vec::new();
    packages_matching(packages, reinstall, "nvidia").await?;
    apt::apt("purge", &["-y"], dbg!(&packages)).await
}

/// Reinstalls NVIDIA-related packages that should be installed
async fn reinstall(packages: &mut Vec<String>) -> anyhow::Result<()> {
    packages.push("nvidia-driver-440".into());
    apt::apt("install", &["-y"], &packages).await
}

/// Whitelist of packages which should be reinstalled
fn reinstall_package(package: &str) -> bool {
    const KEEP: &[&str] = &["nvidia-container-runtime", "system76-driver-nvidia"];

    KEEP.contains(&package)
}

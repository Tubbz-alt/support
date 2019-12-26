use anyhow::Context;
use as_result::MapResult;
use pidfd::PidFd;
use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

/// Applies the repairs for this subcommand
pub async fn repair() -> anyhow::Result<()> {
    // TODO: Apt sources fixes

    info!("fixing broken packages");
    apt("install", &[], &["-f"]).await.context("failed to repair broken packages")?;

    info!("configuring packages");
    dpkg("--configure", &[], &["-a"]).await.context("failed to configure packages")
}

/// Creates an async apt child process
pub async fn apt<S: AsRef<OsStr>>(sub: &str, flags: &[&str], args: &[S]) -> anyhow::Result<()> {
    let mut command = cmd("apt");

    command.arg(sub).args(flags);

    for arg in args {
        command.arg(arg.as_ref());
    }

    command
        .spawn()
        .map(PidFd::from)
        .context("failed to spawn apt command")?
        .into_future()
        .await
        .map_result()
        .context("apt command exited with bad status")
}

/// Creates an async dpkg child process
pub async fn dpkg<S: AsRef<OsStr>>(sub: &str, flags: &[&str], args: &[S]) -> anyhow::Result<()> {
    let mut command = cmd("dpkg");

    command.arg(sub).args(flags);

    for arg in args {
        command.arg(arg.as_ref());
    }

    command
        .spawn()
        .map(PidFd::from)
        .context("failed to spawn dpkg command")?
        .into_future()
        .await
        .map_result()
        .context("dpkg command exited with bad status")
}

/// Creates a command with `DEBIAN_FRONTEND` set to `noninteractive`
fn cmd(command: &str) -> Command {
    let mut cmd = Command::new(command);
    cmd.env("DEBIAN_FRONTEND", "noninteractive").stdin(Stdio::null());
    cmd
}

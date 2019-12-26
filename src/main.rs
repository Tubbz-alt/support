#[macro_use]
extern crate anyhow;

use clap::{App, AppSettings, SubCommand};
use fern::{Dispatch, InitError};
use futures::executor;
use std::{io, process::exit};
use system76_support::*;

fn main() {
    better_panic::install();
    executor::block_on(async move {
        if let Err(why) = main_().await {
            eprintln!("{:#?}", why);
            exit(1)
        }
    });
}

async fn main_() -> anyhow::Result<()> {
    if unsafe { libc::getuid() != 0 } {
        return Err(anyhow!("root is required for this operation"));
    }

    if let Err(why) = install_logger() {
        eprintln!("failed to set up logging: {}", why);
    }

    let matches = App::new("system76-support")
        .about("System76 support utility")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("logs").about("generates logs for the support team"))
        .subcommand(
            SubCommand::with_name("repair")
                .setting(AppSettings::SubcommandRequired)
                .about("common routines for repairing system issues")
                .subcommand(SubCommand::with_name("apt").about("fix common apt errors"))
                .subcommand(SubCommand::with_name("nvidia").about("reinstall NVIDIA drivers")),
        )
        .get_matches();

    match matches.subcommand() {
        ("logs", _) => logs::generate().await,
        ("repair", Some(matches)) => match matches.subcommand() {
            ("apt", _) => repair::apt::repair().await,
            ("nvidia", _) => repair::nvidia::repair().await,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn install_logger() -> Result<(), InitError> {
    Dispatch::new()
        .level(log::LevelFilter::Off)
        .level_for("system76_support", log::LevelFilter::Info)
        .format(move |out, message, _record| out.finish(format_args!("{}", message)))
        .chain(io::stderr())
        .apply()?;

    Ok(())
}

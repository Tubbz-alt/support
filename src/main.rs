#[macro_use]
extern crate anyhow;

use clap::{App, AppSettings, SubCommand};
use std::process::exit;

// NOTE: use async_std::task::block_on as soon as it supports process::Command.
use tokio::runtime::current_thread::Runtime;

fn main() {
    Runtime::new().unwrap().block_on(async move {
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

    let matches = App::new("system76-support")
        .about("System76 support utility")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("logs").about("generates logs for the support team"))
        .get_matches();

    match matches.subcommand() {
        ("logs", _) => system76_support::generate_logs().await,
        _ => unreachable!(),
    }
}

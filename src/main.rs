#[macro_use]
extern crate anyhow;

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

    system76_support::generate_logs().await
}

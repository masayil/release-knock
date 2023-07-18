use crate::server;
use chrono::offset::Local;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use log::info;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the release knock server
    Server(server::Command),
}

pub fn init_log() {
    let env = Env::default()
        .default_filter_or("info")
        .default_write_style_or("never");
    Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [caller:\"{}:{}\"] {}",
                record.level(),
                Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                record.file().unwrap_or("unknown caller file"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}

pub async fn run() -> anyhow::Result<()> {
    let opt = Cli::parse();
    match opt.command {
        Commands::Server(command) => {
            info!("server cli set: {:?}", command);
            server::graceful_shutdown(command).await?;
        }
    }
    Ok(())
}

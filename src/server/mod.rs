mod alert;
mod config;
mod watch;
use crate::{
    common::{http, is_config_exist},
    params::RELEASE_ALERT_CHANNEL,
};
use anyhow::Context;
use clap::Args;
use log::info;
use microkv::MicroKV;
use std::{fs, path::PathBuf};
use tokio::{sync::mpsc, time::Duration};
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

#[derive(Args, Debug)]
pub struct Command {
    /// Sets a custom config file(required)
    #[arg(long, required = true, value_parser = is_config_exist)]
    pub config_file: PathBuf,
}

impl Command {
    fn init(&self) -> anyhow::Result<config::ServerConfig> {
        info!("use config file: {}", self.config_file.display());
        let content =
            fs::read_to_string(&self.config_file).context("Cannot read the config file")?;
        let config: config::ServerConfig =
            serde_json::from_str(&content).context("Fail to deserialize config file")?;
        Ok(config)
    }
}

pub async fn graceful_shutdown(command: Command) -> anyhow::Result<()> {
    Toplevel::new()
        .start("server", |sub_sys| start(sub_sys, command))
        .catch_signals()
        .handle_shutdown_requests(Duration::from_secs(60))
        .await
        .map_err(Into::into)
}

async fn start(sub_sys: SubsystemHandle, command: Command) -> anyhow::Result<()> {
    let config = command.init()?;
    let db = MicroKV::open_with_base_path("github-release", config.db_path)
        .context("Failed to create MicroKV from a stored file or create MicroKV for this file")?
        .set_auto_commit(true);
    let github_api_headers = http::get_github_api_headers(config.github_authorization_token)?;
    let (release_tx, release_rx) = mpsc::channel(RELEASE_ALERT_CHANNEL);
    sub_sys.start("alert", |alert_child| {
        alert::start_alert_module(alert_child, config.alert, release_rx)
    });
    sub_sys.start("watch", move |watch_child| {
        watch::start_watch_module(
            watch_child,
            release_tx,
            db,
            github_api_headers,
            config.period,
            config.retry_interval,
            config.repo_list,
        )
    });
    sub_sys.on_shutdown_requested().await;
    Ok(())
}

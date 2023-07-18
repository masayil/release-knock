mod slack;
mod wechat;
use crate::params::{db, ALERT_SEND_WORKERS};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, Semaphore};
use tokio_graceful_shutdown::SubsystemHandle;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AlertConfig {
    pub slack: slack::AlertProvider,
    pub wechat: wechat::AlertProvider,
}

pub async fn start_alert_module(
    sub_sys: SubsystemHandle,
    alert_config: AlertConfig,
    release_rx: Receiver<db::Release>,
) -> anyhow::Result<()> {
    info!("Alert module is running");
    tokio::join!(
        sub_sys.on_shutdown_requested(),
        alert_spawn_task(alert_config, release_rx)
    );
    info!("Alert module stopped");
    Ok(())
}

async fn alert_spawn_task(alert_config: AlertConfig, mut release_rx: Receiver<db::Release>) {
    let semaphore = Arc::new(Semaphore::new(ALERT_SEND_WORKERS));
    let alert_config = Arc::new(alert_config);
    while let Some(new_release) = release_rx.recv().await {
        let semaphore = Arc::clone(&semaphore);
        let alert_config = Arc::clone(&alert_config);
        tokio::spawn(async move {
            if semaphore.acquire().await.is_ok() {
                handle_task(alert_config, new_release).await;
            } else {
                warn!("The alert_send_workers semaphore has been closed");
                warn!("{:?} sends failed", new_release);
            }
        });
    }
    loop {
        if semaphore.available_permits() == ALERT_SEND_WORKERS {
            debug!("All alert spawn task has stopped.");
            semaphore.close();
            break;
        }
    }
}

async fn handle_task(alert_config: Arc<AlertConfig>, new_release: db::Release) {
    if !alert_config.slack.webhook_url.is_empty() {
        match alert_config.slack.send(new_release.clone()).await {
            Ok(_) => {
                info!("Send alert to slack. RepoName: {}", new_release.name);
            }
            Err(e) => {
                error!(
                    "Fail to send alert to slack. RepoName: {}. Error: {}",
                    new_release.name, e
                );
            }
        }
    }
    if !alert_config.wechat.webhook_url.is_empty() {
        match alert_config.wechat.send(new_release.clone()).await {
            Ok(_) => {
                info!("Send alert to wechat. RepoName: {}", new_release.name);
            }
            Err(e) => {
                error!(
                    "Fail to send alert to wechat. RepoName: {}. Error: {}",
                    new_release.name, e
                );
            }
        }
    }
}

mod task;
use crate::{
    params::{db, db::DbKeyFlag, WATCH_PULL_TIMEOUT},
    server::config::Repo,
};
use anyhow::{anyhow, Context};
use log::{debug, info};
use microkv::MicroKV;
use reqwest::{self, header::HeaderMap, Client};
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc::Sender, time::Duration};
use tokio_graceful_shutdown::SubsystemHandle;

#[derive(Serialize, Deserialize, Clone)]
pub struct Worker {
    pub retry_interval: u64,
    pub repo: Repo,
    pub db: MicroKV,
    pub retry: u8,
}

impl Worker {
    pub fn new(db: MicroKV, retry_interval: u64, repo: Repo, retry: u8) -> Self {
        Worker {
            retry_interval,
            repo,
            db,
            retry,
        }
    }

    fn update_retry_times(&mut self) {
        self.retry += 1;
    }

    async fn handle_new_release(
        &self,
        release_tx: Sender<db::Release>,
        headers: HeaderMap,
    ) -> anyhow::Result<()> {
        let client = Client::builder()
            .timeout(Duration::from_secs(WATCH_PULL_TIMEOUT))
            .user_agent("masayil")
            .default_headers(headers)
            .build()?;

        let resp = client.get(&self.repo.url).send().await?.text().await?;
        let detail: db::ReleaseDetail =
            serde_json::from_str(resp.as_str()).context("Deserialize http response failed!")?;

        let release = db::Release::new(self.repo.url.clone(), self.repo.name.clone(), detail);
        match db::key_in_db_status(self.db.clone(), self.repo.name.as_str()) {
            DbKeyFlag::Exist => {
                let value: db::Release = self.db.get_unwrap(&self.repo.name)?;
                if value != release {
                    release_tx.send(release.clone()).await?;
                    info!(
                        "Send {} latest release version to the alert channel",
                        self.repo.name
                    );
                    info!(
                        "Repo: {} found the new release version. Current version is {}. The latest version is {}",
                        self.repo.name,value.detail.release_name, release.detail.release_name
                    );
                    self.db.put(&self.repo.name, &release)?;
                } else {
                    info!(
                        "Repo: {} has not the new release version. Current version is {}",
                        self.repo.name, value.detail.release_name
                    );
                }
            }
            DbKeyFlag::NotExist => {
                info!(
                    "Repo: {} found the new release version. The latest version is {}",
                    self.repo.name, release.detail.release_name
                );
                self.db.put(&self.repo.name, &release)?;
            }
            DbKeyFlag::FnFail => {
                return Err(anyhow!("Query key: {} in the db failed.", self.repo.name));
            }
        }
        Ok(())
    }
}

pub async fn start_watch_module(
    sub_sys: SubsystemHandle,
    release_tx: Sender<db::Release>,
    db: MicroKV,
    headers: HeaderMap,
    period: u64,
    retry_interval: u64,
    repo_list: Vec<Repo>,
) -> anyhow::Result<()> {
    info!("Watch module is running");
    for v in repo_list.into_iter() {
        let worker = Worker::new(db.clone(), retry_interval, v, 0);
        let headers = headers.clone();
        let release_tx = release_tx.clone();
        sub_sys.start(worker.repo.name.clone().as_str(), move |child| {
            debug!("Start a new watch spawn task for {}", worker.repo.name);
            task::watch_spawn_task(child, worker, headers, release_tx, period)
        });
    }
    sub_sys.on_shutdown_requested().await;
    info!("Watch module stopped");
    Ok(())
}

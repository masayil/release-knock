use super::Worker;
use crate::params::db;
use log::{debug, error};
use reqwest::header::HeaderMap;
use tokio::{
    sync::mpsc::Sender,
    time::{self, Duration, Instant, MissedTickBehavior},
};
use tokio_graceful_shutdown::SubsystemHandle;

pub async fn watch_spawn_task(
    sub_sys: SubsystemHandle,
    worker: Worker,
    headers: HeaderMap,
    release_tx: Sender<db::Release>,
    period: u64,
) -> anyhow::Result<()> {
    let mut intv = time::interval_at(
        Instant::now() + Duration::from_secs(15),
        Duration::from_secs(period),
    );
    let _ = intv.set_missed_tick_behavior(MissedTickBehavior::Delay);
    while !sub_sys.is_shutdown_requested() {
        tokio::select! {
            _ = intv.tick() =>{handle_task(worker.clone(), headers.clone(), release_tx.clone()).await; }
            _ = sub_sys.on_shutdown_requested() =>{}
        }
    }
    debug!("The watch spawn task for {} stopped.", worker.repo.name);
    Ok(())
}

async fn handle_task(worker: Worker, headers: HeaderMap, release_tx: Sender<db::Release>) {
    let result = worker.handle_new_release(release_tx, headers).await;
    if let Err(e) = result {
        error!("Get {} release info failed. Error: {}", worker.repo.name, e);
    }
}

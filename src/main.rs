use log::{error, info};
use release_knock::cli;
use std::process;

#[tokio::main]
async fn main() {
    cli::init_log();
    info!("Start the release-knock server.");
    if let Err(e) = cli::run().await {
        error!("Error: {:?}. shutting down ...", e);
        process::exit(1);
    }
    info!("Stopped the release-knock server.");
}

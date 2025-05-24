pub mod app;
pub mod config;
pub mod log;

use app::{core::crawl, errors::AppError};
use config::setup;
use tokio::signal::ctrl_c;

async fn run() -> Result<(), AppError> {
    let (args, client) = setup().await?;

    tokio::select! {
        biased;

        _ = ctrl_c() => {
            info!("Shutting down: received keyboard interrupt");
        },
        _ = async {
            crawl(&args, &client).await?;
            Ok::<(), AppError>(())
        } => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!("{}", err);
    }
}

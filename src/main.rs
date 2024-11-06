mod resp;

use std::time::Duration;

use resp::constants::DATA_SAVE_INTERVAL_SECS;
use resp::datastore;
use resp::server::run_server;

#[tokio::main]
async fn main() {
    let load_res = datastore::load();
    if let Err(e) = load_res {
        eprintln!("Failed to load data: {}", e);
    }
    // save at an interval in a separate thread
    tokio::spawn(async move {
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(DATA_SAVE_INTERVAL_SECS));
            datastore::save().unwrap_or_else(|e| eprintln!("Failed to save data: {}", e));
        }
    });

    run_server().await.unwrap();
}

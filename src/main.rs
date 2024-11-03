mod resp;

use std::thread;
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
    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(DATA_SAVE_INTERVAL_SECS));
        datastore::save().unwrap_or_else(|e| eprintln!("Failed to save data: {}", e));
    });

    match run_server().await {
        Ok(_) => println!("Server stopped"),
        Err(e) => eprintln!("Server failed: {}", e),
    }
}

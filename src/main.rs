mod resp;

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use resp::commanline::handle_input_cmd;
use resp::constants::DATA_SAVE_INTERVAL_SECS;
use resp::datastore;

fn main() {
    let load_res = datastore::load();
    if let Err(e) = load_res {
        eprintln!("Failed to load data: {}", e);
    }
    // save at an interval in a separate thread
    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(DATA_SAVE_INTERVAL_SECS));
        datastore::save().unwrap_or_else(|e| eprintln!("Failed to save data: {}", e));
    });
    
    loop {
        print!("redis-cli> ");
        io::stdout().flush().unwrap();

        // Read input from the user
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                handle_input_cmd(input);
            }
            Err(e) => {
                eprintln!("Failed to read input: {}", e);
            }
        }
    }
}

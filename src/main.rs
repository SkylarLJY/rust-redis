mod resp;

use std::io::{self, Write};

use resp::commanline::handle_input_cmd;

fn main() {
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
            },
            Err(e) => {
                eprintln!("Failed to read input: {}", e);
            }
        }
    }
}
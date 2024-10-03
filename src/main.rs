mod resp;

use std::io::{self, Write};
use resp::commands::RedisCommand;

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

fn handle_input_cmd(input: &str) {
    let cmd = input.split_whitespace().collect::<Vec<&str>>();

    let resp_cmd = RedisCommand::from_str(cmd[0]);
    match resp_cmd {
        RedisCommand::Ping => {
            println!("PONG");
        },
        RedisCommand::Echo => {
            if cmd.len() > 1 {
                cmd[1..].iter().for_each(|s| print!("{} ", s));
                println!();
            } else {
                println!("No message to echo");
            }
        },
        _ => {
            eprintln!("Unknown command: {}", cmd[0]);
        }
    }
}
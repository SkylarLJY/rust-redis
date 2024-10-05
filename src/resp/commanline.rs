use super::commands::RedisCommand;
use super::datastore::*;


pub fn handle_input_cmd(input: &str) {
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
        RedisCommand::Get => {
            if cmd.len() > 1 {
                match get_value(cmd[1]) {
                    Ok(value) => println!("{}", value),
                    Err(e) => eprintln!("Failed to get value: {}", e),
                }
            } else {
                println!("No key provided to GET");
            }
        },
        RedisCommand::Set => {
            if cmd.len() > 2 {
                match set_value(cmd[1].to_string(), cmd[2].to_string()) {
                    None => println!("OK"),
                    Some(e) => eprintln!("Failed to set value: {}", e),
                }
            } else {
                println!("No key-value pair provided to SET");
            }
        },
        _ => {
            eprintln!("Unknown command: {}", cmd[0]);
        }
    }
}


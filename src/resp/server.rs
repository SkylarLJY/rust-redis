use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use crate::resp::{
    commands::handle_input_cmd, deserialize::deserialize_array, redisconfig, resp_value::RespType,
};

use super::errors::ServerError;

pub fn run_server() -> Result<(), ServerError> {
    let port = redisconfig::get_config("port")
        .unwrap_or("6379".to_string())
        .parse()
        .unwrap();
    let socket_sddr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(socket_sddr).map_err(|_| ServerError::AcceptError)?;
    println!("Server listening on port {}", port);
    loop {
        for stream in listener.incoming() {
            thread::spawn(|| {
                new_connection_handler(stream);
            });
        }
    }
}

fn read_stream(stream: &mut std::net::TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = [4; 1024];
    stream.read(&mut buf)?;
    Ok(buf.to_vec())
}

fn reply(mut s: &mut TcpStream) -> Result<String, ServerError> {
    let input_byte = read_stream(&mut s).map_err(|_| ServerError::ReadError)?;
    let input_array = deserialize_array(input_byte.as_slice())
        .map_err(|_| ServerError::RespParseError(String::from_utf8(input_byte).unwrap()))?;
    match input_array {
        RespType::Array(arr) => {
            let bulk_string_arr = arr.unwrap();
            if bulk_string_arr.len() == 0 {
                return Ok("".to_string());
            }

            let byte_arr: Vec<&str> = bulk_string_arr
                .iter()
                .map(|bs| match bs {
                    RespType::BulkString(Some(bs)) => std::str::from_utf8(bs.as_slice()).unwrap(),
                    _ => "",
                })
                .collect();
            let res = handle_input_cmd(byte_arr).map_err(|e| ServerError::UserInputError(e))?;
            Ok(res)
        }
        _ => Err(ServerError::TypeError),
    }
}

fn new_connection_handler(connection_stream: Result<TcpStream, std::io::Error>) {
    match connection_stream {
        Ok(mut s) => loop {
            let res = reply(&mut s);
            match res {
                Ok(res) => {
                    let resp_res = RespType::SimpleString(res).serialize();
                    s.write_all(&resp_res).unwrap();
                }
                Err(e) => {
                    let error = RespType::Error(e.to_string()).serialize();
                    s.write_all(error.as_slice()).unwrap();
                    match e {
                        ServerError::ReadError => {
                            eprintln!("Failed to read from stream: {}", e);
                        }
                        ServerError::RespParseError(s) => {
                            eprintln!("Failed to parse RESP: {}", s);
                        }
                        ServerError::UserInputError(err) => {
                            eprintln!("Failed to handle user input: {}", err);
                        }
                        ServerError::TypeError => {
                            eprintln!("Failed to handle user input: {}", e);
                        }
                        _ => {
                            eprintln!("Failed to reply request: {}", e);
                        }
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to accept connection: {}", e);
        }
    };
}

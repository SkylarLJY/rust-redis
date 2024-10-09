use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
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
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let stream_read_res: usize = stream.read(&mut buf)?;
    if stream_read_res == 0 {
        return Ok(vec![]);
    }
    Ok(buf.to_vec())
}

fn byte_vec_to_string(byte_vec: Vec<u8>) -> String {
    match String::from_utf8(byte_vec.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to convert byte vec to string: {}", e);
            "ERR".to_string()
        }
    }
}

fn reply(mut s: &mut TcpStream) -> Result<RespType, ServerError> {
    let input_byte = read_stream(&mut s).map_err(|_| ServerError::ReadError)?;
    // no more data instream indicates user has quited connection
    if input_byte.is_empty() {
        return Ok(RespType::Quit);
    }
    let input_array = deserialize_array(input_byte.as_slice())
        .map_err(|_| ServerError::RespParseError(byte_vec_to_string(input_byte)))?;
    match input_array {
        RespType::Array(arr) => {
            let bulk_string_arr = arr.unwrap();
            if bulk_string_arr.len() == 0 {
                // TODO: handle empty array
                return Ok(RespType::SimpleString("".to_string()));
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
                Ok(RespType::Quit) => {
                    break;
                }
                Ok(res) => {
                    let resp_res = res.serialize();
                    s.write_all(&resp_res).unwrap();
                    // s.flush().unwrap();
                }
                Err(e) => {
                    let error = e.to_string();
                    match e {
                        ServerError::ReadError => {
                            eprintln!("{}", e);
                        }
                        ServerError::RespParseError(s) => {
                            eprintln!("{}", s);
                        }
                        ServerError::UserInputError(err) => {
                            eprintln!("{}", err);
                        }
                        ServerError::TypeError => {
                            eprintln!("Failed to handle user input: {}", e);
                        }
                        _ => {
                            eprintln!("Failed to reply request: {}", e);
                        }
                    };
                    let error = RespType::Error(error).serialize();
                    match s.write_all(error.as_slice()) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Failed to write error to stream: {}", e);
                            break;
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

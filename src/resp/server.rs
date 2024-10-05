use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::resp::{
    commands::handle_input_cmd, deserialize::deserialize_array, resp_value::RespType,
};

use super::{constants::TCP_PORT, errors::ServerError};

pub fn run_server() -> Result<(), ServerError> {
    let socket_sddr = SocketAddr::from(([127, 0, 0, 1], TCP_PORT));
    let listener = TcpListener::bind(socket_sddr).map_err(|_| ServerError::AcceptError)?;
    println!("Server listening on port {}", TCP_PORT);
    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let res = handle_stream(&mut s);
                    match res {
                        Ok(res) => {
                            let resp_res = RespType::SimpleString(res).serialize();
                            s.write_all(&resp_res).unwrap();
                        }
                        Err(e) => {
                            eprintln!("Failed to handle stream: {}", e);
                            let err = RespType::Error(e.to_string()).serialize();
                            s.write_all(err.as_slice()).unwrap();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

fn read_stream(stream: &mut std::net::TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = [4; 1024];
    stream.read(&mut buf)?;
    Ok(buf.to_vec())
}

fn handle_stream(mut s: &mut TcpStream) -> Result<String, ServerError> {
    let input_byte = read_stream(&mut s).map_err(|_| ServerError::ReadError)?;
    let input_array =
        deserialize_array(input_byte.as_slice()).map_err(|_| ServerError::RespParseError)?;
    match input_array {
        RespType::Array(arr) => {
            let bulk_string_arr = arr.ok_or(ServerError::RespParseError)?;
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
        _ => {Err(ServerError::TypeError)}
    }
}

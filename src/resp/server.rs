use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error},
    net::{TcpListener, TcpStream},
};

use crate::resp::{
    commands::handle_input_cmd, deserialize::deserialize_array, redisconfig, resp_value::RespType,
};

use super::errors::ServerError;

pub async fn run_server() -> Result<(), Error> {
    let port = redisconfig::get_config("port")
        .unwrap_or("6379".to_string())
        .parse()
        .unwrap();
    let socket_sddr = SocketAddr::from(([127, 0, 0, 1], port));
    // let listener = TcpListener::bind(socket_sddr).map_err(|_| ServerError::AcceptError)?;
    let listener = TcpListener::bind(socket_sddr).await?;
    println!("Server listening on port {}", port);
    loop {
        let (stream, _) = listener.accept().await?;
        match process(stream).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error processing stream: {}", e),
        }
    }
}

async fn process(mut stream: TcpStream) -> Result<(), ServerError> {
    stream.readable().await.unwrap();

    let mut buf = [0u8; 1024];
    let content_len = stream
        .read(&mut buf)
        .await
        .map_err(|_| ServerError::ReadError)?;
    if content_len == 0 {
        return Ok(());
    }

    let input_arr = deserialize_array(&buf[..content_len]).map_err(|_| {
        ServerError::RespParseError("Failed to deserialize input array".to_string())
    })?;
    let res = reply(input_arr)?;

    stream.write(&res.serialize()).await.unwrap();
    Ok(())
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

fn reply(arr: RespType) -> Result<RespType, ServerError> {
    match arr {
        RespType::Array(arr) => {
            if arr.is_none() {
                return Ok(RespType::Array(None));
            }
            let bulk_string_arr = arr.unwrap();
            if bulk_string_arr.len() == 0 {
                // TODO: handle empty array
                return Ok(RespType::SimpleString("".to_string()));
            }

            let str_arr: Vec<&str> = bulk_string_arr
                .iter()
                .map(|bs| match bs {
                    RespType::BulkString(Some(bs)) => std::str::from_utf8(bs.as_slice()).unwrap(),
                    _ => "",
                })
                .collect();
            let res = handle_input_cmd(str_arr).map_err(|e| ServerError::UserInputError(e))?;
            Ok(res)
        }
        _ => Err(ServerError::TypeError),
    }
}

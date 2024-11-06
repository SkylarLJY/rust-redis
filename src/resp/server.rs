use std::{net::SocketAddr, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error},
    net::{TcpListener, TcpStream},
    time::timeout,
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
    let listener = TcpListener::bind(socket_sddr).await?;
    println!("Server listening on port {}", port);
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

pub async fn process(mut stream: TcpStream) {
    // use loop to continue processing requests from the same client
    loop {
        // put a timeout on the read operation
        let stream_ready = timeout(Duration::from_secs(1), stream.readable());
        match stream_ready.await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("Failed to check if stream is readable: {}", e);
                break;
            }
            Err(_) => {
                println!("Timeout waiting for data");
                break;
            }
        };
        let mut buf = [0u8; 1024];
        let content_len = match stream.read(&mut buf).await {
            Ok(content_len) => content_len,
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                0
            }
        };
        if content_len == 0 {
            break;
        }

        let input_arr = deserialize_array(&buf[..content_len])
            .map_err(|_| {
                ServerError::RespParseError("Failed to deserialize input array".to_string())
            })
            .unwrap();
        let res = reply(input_arr).unwrap();

        stream.writable().await.unwrap();
        stream.write(&res.serialize()).await.unwrap();
        stream.flush().await.unwrap();
    }
}

fn reply(arr: RespType) -> Result<RespType, ServerError> {
    match arr {
        RespType::Array(arr) => {
            if arr.is_none() {
                return Ok(RespType::Null);
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

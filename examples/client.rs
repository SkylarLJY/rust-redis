use bytes::Bytes;
use mini_redis::{client, Result};
use redis_server::resp::commands::RedisCommand;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() -> Result<()> {
    // send from all task to the manager task
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        // a channel for the response of the set command
        let (res_tx, res_rx) = oneshot::channel();
        let cmd = ClientCommand {
            cmd: RedisCommand::Get("123".to_string()),
            res_channel: res_tx,
        };
        tx.send(cmd).await.unwrap();
        let res = res_rx.await.unwrap();
        println!("[main] GET = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();
        let cmd = ClientCommand {
            cmd: RedisCommand::Set("hello".to_string(), "world".to_string(), vec![]),
            res_channel: res_tx,
        };
        tx2.send(cmd).await.unwrap();
        let res = res_rx.await.unwrap();
        println!("[main] SET = {:?}", res);
    });

    let clientManager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd.cmd {
                RedisCommand::Get(key) => match client.get(&key).await {
                    Ok(val) => {
                        println!("[manager] GET = {:?}", val);
                        let _ = cmd.res_channel.send(ServerResponse::Value(val));
                    }
                    Err(e) => {
                        let _ = cmd
                            .res_channel
                            .send(ServerResponse::Error("Key not found".to_string()));
                    }
                },
                RedisCommand::Set(key, val, _) => {
                    let _ = client
                        .set(&key, Bytes::from(val.clone().into_bytes()))
                        .await;
                    let _ = cmd.res_channel.send(ServerResponse::None);
                }
                _ => {}
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    clientManager.await.unwrap();

    Ok(())
}

struct ClientCommand {
    cmd: RedisCommand,
    res_channel: oneshot::Sender<ServerResponse>,
}

#[derive(Debug)]
enum ServerResponse {
    Value(Option<Bytes>),
    Error(String),
    None,
}

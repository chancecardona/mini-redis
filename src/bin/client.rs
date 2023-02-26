use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        val: Bytes,
    }
}

#[tokio::main]
async fn main() {
    // Create a multiple input multiple output channel for message passing
    // between threads
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    // Create a single input single output channel for the response
    //let (tx, rx) = oneshot::channel();
    
    // Manager task to recv and process messages from the channel
    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key } => {
                    // Get key by ref since it's a String
                    client.get(&key).await;
                }
                Command::Set { key, val } => {
                    client.set(&key, val).await;
                }
            }
        }
    });

    // Task to get the key
    let t1 = tokio::spawn(async move {
        let cmd = Command::Get {
            key: "foo".to_string(),
        };

        tx.send(cmd).await.unwrap();
    });
    
    // Task to set the key
    let t2 = tokio::spawn(async move {
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
        };

        tx2.send(cmd).await.unwrap();
    });
   
    // await the join handles to let threads finish
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

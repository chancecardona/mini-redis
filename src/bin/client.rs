use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        // Oneshot sender is part of command so we can send a resp back for the specific command
        resp: Responder<Option<Bytes>>, 
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}
// To send responses back to the requester
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // Create a multiple input multiple output channel for message passing
    // between threads
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    
    // Manager task to recv and process messages from the channel
    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    // Get key by ref since it's a String
                    client.get(&key).await;
                    let _ = resp.send(res); // Ignore Errors
                    // oneshot .send doesn't need await as it either fails or succeeds instantly.
                }
                Command::Set { key, val, resp } => {
                    client.set(&key, val).await;
                    let _ = resp.send(res); // Ignore Errors
                }
            }
        }
    });

    // Task to get the key
    let t1 = tokio::spawn(async move {
        // Create a single input single output channel for the response
        let (resp_tx, resp_rx) = oneshot::channel();
        // Create Command
        // (the resp_tx part of the channel is sent with the commands so that
        // each command has its own oneshot channel for responses about that command)
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        // Send Get Command
        tx.send(cmd).await.unwrap();

        // Await response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });
    
    // Task to set the key
    let t2 = tokio::spawn(async move {
        // Create a single input single output channel for the response
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();

        // Await response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });
   
    // await the join handles to let threads finish
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

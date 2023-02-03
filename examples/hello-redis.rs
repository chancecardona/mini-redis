use mini_redis::{client, Result};

// This is a macro that makes the async fn into a sync fn main
// which init's the runtime and execs the func.
#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis addr
    // this is async, so the await relenquishes ctrl back to the thread.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Sets key "hello" with value "world".
    client.set("hello", "world".into()).await?;
    // Get "hello" key.
    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}

use soltop::{hello_from_lib, get_rpc_url};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let message: String = hello_from_lib();
    println!("{}", message);

    let rpc_url: String = get_rpc_url();
    println!("RPC URL: {}", rpc_url);

    let connected: bool = soltop::rpc::check_connection(&rpc_url).await?;
    println!("Connected: {}", connected);

    Ok(())
}

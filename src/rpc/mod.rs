use anyhow::Result;

pub fn get_rpc_url() -> String {
    "https://api.mainnet-beta.solana.com".to_string()
}

pub async fn check_connection(rpc_url: &str) -> Result<bool> {
    // We'll implement this properly later
    // For now, just return true
    println!("Checking connection to RPC URL: {}", rpc_url);
    Ok(true)
}

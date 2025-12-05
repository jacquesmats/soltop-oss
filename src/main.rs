use anyhow::Result;
use soltop::rpc::RpcClient;
use tokio::join;


#[tokio::main]
async fn main() -> Result<()> {
    // Use public Solana mainnet RPC
    let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
    let client = RpcClient::new(rpc_url);

    println!("Fetching latest slot from Solana mainnet...");
    
    let slot = client.get_latest_slot().await?;
    println!("✓ Current slot: {}", slot);

    println!("\nFetching recent block data...");
    let (block1, block2, block3) = join!(
        client.get_block(slot - 10),
        client.get_block(slot - 11),
        client.get_block(slot - 12),
    );
    
    let blocks = [block1, block2, block3];

    for (i, block) in blocks.iter().enumerate() {
        match block {
            Ok(Some(block_response)) => {
                if let Some(block_data) = &block_response.result {
                    println!("✓ Block {} has {} transactions", i + 1, block_data.transactions.len());
                    
                    if let Some(first_tx) = block_data.transactions.first() {
                        if let Some(program_id) = first_tx.transaction.message.account_keys.first() {
                            println!("  First program ID: {}", program_id);
                        }
                    }
                }
            }
            Ok(None) => println!("✗ Block {} was skipped", i + 1),
            Err(e) => println!("✗ Block {} error: {}", i + 1, e),
        }
    }

    Ok(())
}
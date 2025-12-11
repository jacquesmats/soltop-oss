use anyhow::Result;
use soltop::rpc::RpcClient;
use soltop::stats::NetworkState;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
    let client = RpcClient::new(rpc_url);
    
    // Create network state tracker
    let mut state = NetworkState::new(
        Duration::from_secs(300),  // 5 minute window
        750  // ~5 min of slots
    );
    
    println!("Fetching latest slot...");
    let slot = client.get_latest_slot().await?;
    println!("✓ Current slot: {}\n", slot);
    
    println!("Processing recent blocks...");
    
    // Process 5 recent blocks
    for i in 1..=5 {
        let block_slot = slot - i;
        
        match client.get_block(block_slot).await? {
            Some(block_response) => {
                if let Some(block_data) = block_response.result {
                    state.process_block(block_slot, &block_data);
                    println!("✓ Processed block {} ({} txs)", 
                             block_slot, 
                             block_data.transactions.len());
                }
            }
            None => println!("✗ Block {} was skipped", block_slot),
        }
    }
    
    // Display statistics
    println!("\n=== Program Statistics ===\n");
    println!("{:<45} {:>8} {:>10} {:>12} {:>12} {:>12} {:>12}", 
             "Program", "Txs", "Success%", "CU/s", "Avg CU", "Min CU", "Max CU");
    println!("{}", "─".repeat(115));
    
    let stats = state.get_program_stats();
    for program_stats in stats.iter().take(10) {  // Top 10
        println!("{:<45} {:>8} {:>9.1}% {:>12.0} {:>12.0} {:>12} {:>12}",
                 &program_stats.program_id[..program_stats.program_id.len().min(45)],
                 program_stats.total_transactions(),
                 program_stats.success_rate(),
                 program_stats.cu_per_second(),
                 program_stats.avg_cu_per_transaction(),
                 program_stats.min_cu(),
                 program_stats.max_cu());
    }
    
    println!("\nTracking {} programs total", state.program_count());
    
    Ok(())
}
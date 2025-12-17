use anyhow::Result;
use std::time::Duration;
use clap::Parser;
use soltop::{NetworkMonitor, MonitorConfig};
use tokio;

#[derive(Parser, Debug)]
#[command(name = "soltop")]
#[command(about = "Terminal UI for Solana network monitoring", long_about = None)]
struct Args {
    /// Enable verbose performance statistics
    #[arg(short, long)]
    verbose: bool,
    
    /// RPC endpoint URL
    #[arg(
        long,
        default_value = "https://api.mainnet-beta.solana.com",
        help = "RPC endpoint URL"
    )]
    rpc_url: String,

    /// Hide system programs (Vote, ComputeBudget, System)
    #[arg(long)]
    hide_system: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Create configuration
    let config = MonitorConfig {
        rpc_url: args.rpc_url,
        window_duration: Duration::from_secs(5 * 60),  // 5 minutes
        buffer_capacity: 750,
        poll_interval: Duration::from_millis(400),
    };
    
    // Create monitor
    let monitor = NetworkMonitor::new(config);
    
    // Get shared state for printing stats
    let state = monitor.get_state();
    
    // Spawn stats printer task
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Read current stats
            let state = state.read().await;
            let stats = state.get_program_stats(true);  // Hide system programs
            
            // Print summary
            println!("\n=== soltop - Slot {} ===", state.current_slot);
            println!("Programs tracked: {}", stats.len());
            
            // Print top 5 programs
            for (i, stat) in stats.iter().take(5).enumerate() {
                println!(
                    "{}. {} - {:.0} tx/s, {:.0} CU/s, {:.1}% success",
                    i + 1,
                    &stat.program_id[..8],  // First 8 chars
                    stat.transactions_per_second(),
                    stat.cu_per_second(),
                    stat.success_rate(),
                );
            }
        }
    });
    
    // Start the pipeline (runs forever)
    monitor.start().await?;
    
    Ok(())
}
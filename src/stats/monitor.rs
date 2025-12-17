use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use anyhow::Result;

use crate::rpc::RpcClient;
use super::network::NetworkState;

/// Configuration for the network monitor
pub struct MonitorConfig {
    pub rpc_url: String,
    pub window_duration: Duration,
    pub buffer_capacity: usize,
    pub poll_interval: Duration,  // How often to fetch new slots
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            window_duration: Duration::from_secs(5 * 60),  // 5 minutes
            buffer_capacity: 750,  // ~5 minutes at 400ms/slot
            poll_interval: Duration::from_millis(400),      // Match slot time
        }
    }
}

/// Main network monitoring coordinator
pub struct NetworkMonitor {
    config: MonitorConfig,
    rpc_client: RpcClient,
    state: Arc<RwLock<NetworkState>>,
}

impl NetworkMonitor {
    /// Create a new network monitor
    pub fn new(config: MonitorConfig) -> Self {
        let rpc_client = RpcClient::new(config.rpc_url.clone());
        
        let state = Arc::new(RwLock::new(
            NetworkState::new(
                config.window_duration,
                config.buffer_capacity,
            )
        ));
        
        Self {
            config,
            rpc_client,
            state,
        }
    }
    
    /// Get a clone of the shared state (for consumers to access)
    pub fn get_state(&self) -> Arc<RwLock<NetworkState>> {
        Arc::clone(&self.state)
    }

    /// Producer task: continuously fetch slots and send to channel
    async fn produce_slots(
        rpc_client: RpcClient,
        poll_interval: Duration,
        tx: mpsc::Sender<u64>,
    ) -> Result<()> {
        let mut current_slot = rpc_client.get_latest_slot().await?;
        println!("Starting from slot: {}", current_slot);
        
        loop {
            // Check where we are
            let latest_slot = rpc_client.get_latest_slot().await?;
            
            if current_slot <= latest_slot {
                // Send slot immediately
                tx.send(current_slot).await?;
                current_slot += 1;
                // No sleep when catching up!
            } else {
                // We're ahead, wait a bit
                tokio::time::sleep(poll_interval).await;
            }
        }
    }

    /// Consumer task: receive slots from channel and update state
    async fn consume_slots(
        state: Arc<RwLock<NetworkState>>,
        rpc_client: RpcClient,
        mut rx: mpsc::Receiver<u64>,
    ) -> Result<()> {
        while let Some(slot) = rx.recv().await {
            match rpc_client.get_block(slot).await {
                Ok(Some(block_response)) if block_response.result.is_some() => {
                    // Happy path: block exists and has data
                    let block_data = block_response.result.unwrap();
                    
                    {  // Explicit scope for lock
                        let mut state = state.write().await;
                        state.process_block(slot, &block_data, false);
                    }  // Lock dropped here
                }
                Ok(_) => {
                    // Block skipped or no data
                }
                Err(e) => {
                    // Network error - log but continue
                    eprintln!("Error fetching slot {}: {}", slot, e);
                }
            }
        }
        
        println!("Consumer shutting down (channel closed)");
        Ok(())
    }

    /// Start the monitoring pipeline
    /// This function runs forever (until Ctrl+C)
    pub async fn start(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel::<u64>(100);
        
        // Clone data for consumer
        let state = Arc::clone(&self.state);
        let rpc_client = RpcClient::new(self.config.rpc_url.clone());
        
        // Clone data for producer (NEW!)
        let producer_client = RpcClient::new(self.config.rpc_url.clone());
        let poll_interval = self.config.poll_interval;
        
        // Spawn producer
        let producer = tokio::spawn(async move {
            // Use cloned data, not self
            if let Err(e) = Self::produce_slots(producer_client, poll_interval, tx).await {
                eprintln!("Producer error: {}", e);
            }
        });
        
        // Spawn consumer
        let consumer = tokio::spawn(async move {
            if let Err(e) = Self::consume_slots(state, rpc_client, rx).await {
                eprintln!("Consumer error: {}", e);
            }
        });
        
        let _ = tokio::join!(producer, consumer);
        Ok(())
    }
}
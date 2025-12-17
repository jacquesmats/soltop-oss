use std::sync::Arc;
use std::cmp::Reverse;
use tokio::sync::RwLock;
use crate::stats::NetworkState;

/// Main TUI application
pub struct App {
    /// Reference to shared network state (updated by NetworkMonitor)
    network_state: Arc<RwLock<NetworkState>>,
    
    /// Whether the app should keep running
    pub running: bool,
    
    /// Currently selected row in the table
    pub selected_row: usize,

    cached_stats: Vec<ProgramStatsDisplay>,
}

impl App {
    /// Create a new App with reference to network state
    pub fn new(network_state: Arc<RwLock<NetworkState>>) -> Self {
        Self {
            network_state,
            running: true,
            selected_row: 0,
            cached_stats: vec![],
        }
    }

    // ← Add this: update cached stats
    pub async fn update_stats(&mut self) {
        self.cached_stats = self.get_stats().await;
    }
    
    // ← Add this: get cached stats for rendering
    pub fn get_cached_stats(&self) -> &[ProgramStatsDisplay] {
        &self.cached_stats
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;
        
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
            }
            KeyCode::Down => {
                // TODO: Move selection down (we'll implement this later)
            }
            KeyCode::Up => {
                // TODO: Move selection up (we'll implement this later)
            }
            _ => {}
        }
    }

    /// Get current network statistics for rendering
    pub async fn get_stats(&self) -> Vec<ProgramStatsDisplay> {
        let state = self.network_state.read().await;

        let mut display = Vec::new();

        for (program_id, stats) in state.programs.iter() {
            display.push(
                ProgramStatsDisplay {
                    program_id: program_id.clone(),
                    tx_per_sec: stats.transactions_per_second(),
                    total_txs: stats.total_transactions(),
                    success_rate: stats.success_rate(),
            });
        }
        
        // Sort by total_txs descending
        display.sort_by_key(|s| Reverse(s.total_txs));
        
        display
    }
}

/// Simplified struct for displaying program stats in UI
/// (We'll expand this as we build the table)
pub struct ProgramStatsDisplay {
    pub program_id: String,
    pub tx_per_sec: f64,
    pub total_txs: u32,
    pub success_rate: f64,
}
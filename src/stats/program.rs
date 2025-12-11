use std::time::Instant;
use super::RingBuffer;

/// Statistics for a single Solana program
pub struct ProgramStats {
    /// The program's public key (e.g., "JUP4Fb2c...")
    pub program_id: String,
    
    /// Ring buffer of slot-level statistics
    /// Each entry = aggregated stats for ONE SLOT
    slot_timeline: RingBuffer<SlotStats>,
    
    /// When we started collecting data for this program
    start_time: Instant,
}

/// Statistics for a single slot
#[derive(Debug, Clone)]
pub struct SlotStats {
    /// When this slot was processed
    pub timestamp: Instant,
    
    /// Total compute units consumed in this slot
    pub total_cu: u64,
    
    /// Number of transactions in this slot
    pub tx_count: u32,
    
    /// Number of successful transactions
    pub success_count: u32,
    
    /// Average CU per transaction (precomputed)
    pub avg_cu: f64,
    
    /// Minimum CU in this slot
    pub min_cu: u64,
    
    /// Maximum CU in this slot
    pub max_cu: u64,
}


impl ProgramStats {
    /// Create new program stats tracker
    pub fn new(program_id: String, capacity: usize) -> Self {
        Self {
            program_id,
            slot_timeline: RingBuffer::new(capacity),
            start_time: Instant::now(),
        }
    }
    
    /// Record statistics for a slot
    pub fn record_slot(&mut self, slot_stats: SlotStats) {
        self.slot_timeline.push(slot_stats);
    }
    
    /// Get total transaction count across all slots in buffer
    pub fn total_transactions(&self) -> u32 {
        self.slot_timeline.iter().map(|s| s.tx_count).sum() 
    }
    
    /// Calculate success rate (0.0 to 100.0)
    pub fn success_rate(&self) -> f64 {
        let success_txs: u32 = self.slot_timeline.iter().map(|s| s.success_count).sum();
        let all_txs = self.total_transactions();

        if all_txs == 0 {
            100.0
        } else {
            (success_txs as f64 / all_txs as f64) * 100.0
        }

    }
    
    /// Calculate transactions per second
    pub fn transactions_per_second(&self) -> f64 {
        if self.slot_timeline.is_empty() {
            return 0.0;
        } else {
            let time_span = self.get_time_span();
            let total_txs = self.total_transactions();

            total_txs as f64 / time_span
        }

    }
    
    /// Calculate compute units per second
    pub fn cu_per_second(&self) -> f64 {
        if self.slot_timeline.is_empty() {
             0.0
        } else {
            let time_span = self.get_time_span();
            let total_cu: u64 = self.slot_timeline.iter().map(|s| s.total_cu).sum();
            
             total_cu as f64 / time_span
              
        }
    }
    
    /// Calculate average CU per transaction across all slots
    pub fn avg_cu_per_transaction(&self) -> f64 {
        if self.slot_timeline.is_empty() {
             0.0
        } else {
            let total_cu: u64 = self.slot_timeline.iter().map(|s| s.total_cu).sum();
            let total_txs = self.total_transactions();

             total_cu as f64 / total_txs as f64
        }
    }
    
    /// Get minimum CU from all slots
    pub fn min_cu(&self) -> u64 {
        self.slot_timeline.iter().map(|s| s.min_cu).min().unwrap_or(0)
    }
    
    /// Get maximum CU from all slots
    pub fn max_cu(&self) -> u64 {
        self.slot_timeline.iter().map(|s| s.max_cu).max().unwrap_or(0)
    }

    // Calculate time passed between oldest and newest timestamps
    fn get_time_span(&self) -> f64 {
        let mut iter = self.slot_timeline.iter();
        let first = iter.next();
        let last = iter.last();
        
        // Calculate duration
        if let (Some(first_slot), Some(last_slot)) = (first, last) {
            let duration = last_slot.timestamp.duration_since(first_slot.timestamp);
            duration.as_secs_f64()
        } else {
            0.0
        }
    }
}
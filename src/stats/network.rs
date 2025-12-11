use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::stats::program::SlotStats;

use crate::rpc::{BlockData, extract_cu, extract_program_id, extract_cu_timed};

use super::ProgramStats;
use std::cmp::Reverse;

/// Network-wide state containing all program statistics
pub struct NetworkState {
    /// Map of program_id -> statistics
    programs: HashMap<String, ProgramStats>,
    
    /// Current slot being processed
    pub current_slot: u64,
    
    /// When we started monitoring
    start_time: Instant,
    
    /// Target window duration (e.g., 5 minutes)
    window_duration: Duration,
    
    /// Ring buffer capacity (e.g., 750 slots for 5 min)
    buffer_capacity: usize,

    /// Performance stats
    pub perf_stats: PerfStats,
}

impl NetworkState {
    /// Create a new network state tracker
    pub fn new(window_duration: Duration, buffer_capacity: usize) -> Self {
        Self { 
            programs: HashMap::new(), 
            current_slot: 0, 
            start_time: Instant::now(), 
            window_duration, 
            buffer_capacity,
            perf_stats: PerfStats::new(),
        }
    }
    
    /// Record a transaction for a specific program
    /// Note: This accumulates data for the current slot
    pub fn record_transaction(&mut self, program_id: String, cu_used: u64, success: bool) {
        let slot_stats = SlotStats{
            timestamp: Instant::now(),
            total_cu: cu_used,
            tx_count: 1,
            success_count: if success { 1 } else { 0 },
            avg_cu: cu_used as f64,
            min_cu: cu_used,
            max_cu: cu_used,
        };

        self.programs
                    .entry(program_id.clone())
                    .or_insert_with(|| ProgramStats::new(program_id, self.buffer_capacity))
                    .record_slot(slot_stats);
    }
    
    /// Update the current slot
    pub fn update_slot(&mut self, slot: u64) {
        self.current_slot = slot;
    }
    
    /// Get statistics for all programs, sorted by transaction count
    pub fn get_program_stats(&self) -> Vec<&ProgramStats> {
        let mut stats: Vec<_> = self.programs.values().collect();

        stats.sort_by_key(|s| Reverse(s.total_transactions()));

        stats
    }
    
    /// Get the actual window duration (min of elapsed time and target window)
    pub fn actual_window(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        std::cmp::min(elapsed, self.window_duration)
    }
    
    /// Get number of programs being tracked
    pub fn program_count(&self) -> usize {
        self.programs.len()
    }

    // Process all transactions in a block
    pub fn process_block(&mut self, slot: u64, block_data: &BlockData, verbose: bool) {
        let start = if verbose { Some(Instant::now()) } else { None };
        
        // Update current slot
        self.update_slot(slot);
        
        // Accumulate per-program statistics for this slot
        // HashMap: program_id -> (total_cu, tx_count, success_count, all_cu_values)
        let mut slot_data: HashMap<String, SlotAccumulator> = HashMap::new();
        
        // Process each transaction and accumulate
        for tx_data in &block_data.transactions {
            if let Some((program_id, cu_used, success)) = self.extract_tx_data(tx_data, verbose) {
                let acc = slot_data.entry(program_id).or_insert_with(SlotAccumulator::new);
                acc.add_transaction(cu_used, success);
            }
        }
        
        // Now convert accumulated data to SlotStats and record
        let timestamp = Instant::now();
        for (program_id, acc) in slot_data {
            let slot_stats = acc.to_slot_stats(timestamp);
            
            // Get or create ProgramStats and record this slot
            self.programs
                .entry(program_id.clone())
                .or_insert_with(|| ProgramStats::new(program_id, self.buffer_capacity))
                .record_slot(slot_stats);
        }

        if let Some(start_time) = start {
            self.perf_stats.process_block_time += start_time.elapsed();
        }
    }
    
    /// Extract relevant data from a transaction
    fn extract_tx_data(&mut self, tx_data: &crate::rpc::TransactionData, verbose: bool) -> Option<(String, u64, bool)> {
        // Extract program ID
        let program_id = extract_program_id(&tx_data)?;
        
        // Check success
        let success = tx_data.meta
            .as_ref()
            .map(|meta| meta.err.is_none())
            .unwrap_or(false);

        // Extract compute units from logs, with verbose/perf_stats tracking if needed
        let total_cu: u64 = tx_data.meta
            .as_ref()
            .and_then(|meta| meta.log_messages.as_ref())
            .map(|logs| {
                logs.iter()
                    .filter_map(|log| {
                        if verbose {
                            let (cu, elapsed) = crate::rpc::extract_cu_timed(log);
                            self.perf_stats.extract_cu_time += elapsed;
                            self.perf_stats.extract_cu_calls += 1;
                            cu
                        } else {
                            extract_cu(log)
                        }
                    })
                    .sum()
            })
            .unwrap_or(0);
        
        Some((program_id, total_cu, success))
    }
}

/// Helper struct to accumulate transaction data for a single slot
struct SlotAccumulator {
    total_cu: u64,
    tx_count: u32,
    success_count: u32,
    cu_values: Vec<u64>,  // To calculate min/max/avg
}

impl SlotAccumulator {
    fn new() -> Self {
        Self {
            total_cu: 0,
            tx_count: 0,
            success_count: 0,
            cu_values: Vec::new(),
        }
    }
    
    fn add_transaction(&mut self, cu_used: u64, success: bool) {
        self.total_cu += cu_used;
        self.tx_count += 1;
        self.cu_values.push(cu_used);   // TO DO: Here we are storing all cu values for this program,
                                        // just to calculate min and max. This can be optimzied. But
                                        // can we do more with this values maybe? p99?

        if success {
            self.success_count += 1;
        }
    }
    
    fn to_slot_stats(self, timestamp: Instant) -> SlotStats {
        // Handle empty case for avg
        let avg_cu = if self.tx_count > 0 {
            self.total_cu as f64 / self.tx_count as f64
        } else {
            0.0
        };

        let min_cu = self.cu_values.iter().copied().min().unwrap_or(0);
        let max_cu = self.cu_values.iter().copied().max().unwrap_or(0);
        
        SlotStats {
            timestamp,
            total_cu: self.total_cu,
            tx_count: self.tx_count,
            success_count: self.success_count,
            avg_cu: avg_cu,
            min_cu: min_cu, 
            max_cu: max_cu,
        }
    }
}

/// Performance statistics (only used in verbose mode)
#[derive(Debug, Default)]
pub struct PerfStats {
    pub process_block_time: Duration,
    pub extract_cu_time: Duration,
    pub extract_cu_calls: u64,
}

impl PerfStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn print_summary(&self, blocks_processed: usize) {
        println!("\n📊 Performance Summary:");
        println!("  Blocks processed: {}", blocks_processed);
        println!("  Total process_block time: {:.2}ms", self.process_block_time.as_secs_f64() * 1000.0);
        println!("  - Avg per block: {:.2}ms", self.process_block_time.as_secs_f64() * 1000.0 / blocks_processed as f64);
        println!("  Total extract_cu time: {:.2}ms", self.extract_cu_time.as_secs_f64() * 1000.0);
        println!("  - extract_cu calls: {}", self.extract_cu_calls);
        println!("  - Avg per extract_cu call: {:.2}µs", 
                 self.extract_cu_time.as_secs_f64() * 1_000_000.0 / self.extract_cu_calls as f64);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_network_state_with_slot_stats() {
        let mut state = NetworkState::new(
            Duration::from_secs(300),  // 5 minutes
            750  // buffer capacity
        );
        
        // Manually create some SlotStats to test
        let slot1 = SlotStats {
            timestamp: Instant::now(),
            total_cu: 100_000,
            tx_count: 2,
            success_count: 2,
            avg_cu: 50_000.0,
            min_cu: 42_000,
            max_cu: 58_000,
        };
        
        let slot2 = SlotStats {
            timestamp: Instant::now(),
            total_cu: 80_000,
            tx_count: 2,
            success_count: 1,
            avg_cu: 40_000.0,
            min_cu: 38_000,
            max_cu: 42_000,
        };
        
        // Record slots for Jupiter
        state.programs
            .entry("JUP4Fb2c".to_string())
            .or_insert_with(|| ProgramStats::new("JUP4Fb2c".to_string(), 750))
            .record_slot(slot1);
            
        state.programs
            .entry("JUP4Fb2c".to_string())
            .or_insert_with(|| ProgramStats::new("JUP4Fb2c".to_string(), 750))
            .record_slot(slot2);
        
        // Check program count
        assert_eq!(state.program_count(), 1);
        
        // Get stats
        let stats = state.get_program_stats();
        assert_eq!(stats.len(), 1);
        
        // Verify Jupiter stats
        let jupiter = stats[0];
        assert_eq!(jupiter.program_id, "JUP4Fb2c");
        assert_eq!(jupiter.total_transactions(), 4);  // 2 + 2
        assert_eq!(jupiter.success_rate(), 75.0);     // 3/4 * 100
        assert_eq!(jupiter.min_cu(), 38_000);
        assert_eq!(jupiter.max_cu(), 58_000);
    }
}
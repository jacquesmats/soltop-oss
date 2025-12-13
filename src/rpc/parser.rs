use regex::Regex;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Regex to match compute unit consumption in logs
/// Matches: "Program XXX consumed 12345 of 200000 compute units"
static CU_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Program (\S+) consumed (\d+) of \d+ compute units").unwrap()
});

/// Extract program ID and CU consumption from a log message
/// Returns None if the log doesn't contain CU information
fn parse_program_cu(log: &str) -> Option<(String, u64)> {
    let caps = CU_REGEX.captures(log)?;
    let program_id = caps.get(1)?.as_str().to_string();
    let cu = caps.get(2)?.as_str().parse().ok()?;
    Some((program_id, cu))                // Option<u64>
}

/// Extract ALL programs and their TOTAL CU consumption from logs
/// 
/// If a program appears multiple times, CU values are summed.
pub fn extract_program_cu(logs: &[String]) -> HashMap<String, u64> {
    let mut programs = HashMap::new();
    
    for log in logs {
        if let Some((program_id, cu)) = parse_program_cu(log) {
            *programs.entry(program_id).or_insert(0) += cu;
        }
    }
    
    programs
}

// timed version
pub fn extract_program_cu_timed(logs: &[String]) -> (HashMap<String, u64>, Duration) {
    let start = Instant::now();
    let result = extract_program_cu(logs);
    let elapsed = start.elapsed();
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_program_extraction() {
        let logs = vec![
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3158 of 200207 compute units".to_string(),
            "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 7913 of 204938 compute units".to_string(),
        ];

        let programs = extract_program_cu(&logs);
        
        assert_eq!(programs.len(), 2);
        assert_eq!(programs.get("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), Some(&3158));
        assert_eq!(programs.get("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"), Some(&7913));
    }

    #[test]
    fn test_sum_multiple_invocations() {
        let logs = vec![
            "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 7913 of 204938 compute units".to_string(),
            "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 199 of 50825 compute units".to_string(),
            "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 consumed 135734 of 179060 compute units".to_string(),
        ];

        let programs = extract_program_cu(&logs);
        
        // Should sum all three: 7913 + 199 + 135734 = 143846
        assert_eq!(programs.get("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"), Some(&143846));
    }
}
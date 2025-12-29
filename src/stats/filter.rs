/// Well-known Solana system programs that create noise in statistics
pub const SYSTEM_PROGRAMS: &[&str] = &[
    "Vote111111111111111111111111111111111111111",
    "ComputeBudget111111111111111111111111111111",
    "11111111111111111111111111111111", // System Program
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", // Token 2022
];

/// Check if a program ID is a system program
pub fn is_system_program(program_id: &str) -> bool {
    SYSTEM_PROGRAMS.contains(&program_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_program() {
        assert!(is_system_program(
            "Vote111111111111111111111111111111111111111"
        ));
        assert!(!is_system_program(
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        ));
    }
}

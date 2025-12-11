use regex::Regex;
use once_cell::sync::Lazy;

/// Regex to match compute unit consumption in logs
/// Matches: "Program XXX consumed 12345 of 200000 compute units"
static CU_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"consumed (\d+) of \d+ compute units").unwrap()
});

/// Extract compute units from a log message
/// Returns None if the log doesn't contain CU information
pub fn extract_cu(log: &str) -> Option<u64> {
    CU_REGEX.captures(log)?    // Option<Captures> - early return if None
            .get(1)?               // Option<Match> - early return if None  
            .as_str() 
            .parse()  
            .ok()                  // Option<u64>
}

/// Extract program ID from transaction
pub fn extract_program_id(tx_data: &crate::rpc::TransactionData) -> Option<String> {
    let index = tx_data.transaction
                .message
                .instructions.first()?
                .program_id_index;
    
    tx_data.transaction.message.account_keys.get(index as usize).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_cu() {
        let log = "Program JUP4Fb2c consumed 42069 of 200000 compute units";
        assert_eq!(extract_cu(log), Some(42069));
        
        let log2 = "Program log: Some other message";
        assert_eq!(extract_cu(log2), None);
    }
    
    #[test]
    fn test_extract_program_id() {
        use crate::rpc::types::{TransactionData, Transaction, Message, Instruction, TransactionMeta};
        
        // Create a mock transaction with the Vote program
        let tx_data = TransactionData {
            transaction: Transaction {
                message: Message {
                    account_keys: vec![
                        "FeePayerWallet1111111111111111111111111111".to_string(),
                        "VoteAccount11111111111111111111111111111111".to_string(),
                        "Vote111111111111111111111111111111111111111".to_string(),
                    ],
                    instructions: vec![
                        Instruction {
                            program_id_index: 2,  // Points to the Vote program
                        }
                    ],
                },
            },
            meta: None,
        };
        
        let program_id = extract_program_id(&tx_data);
        assert_eq!(program_id, Some("Vote111111111111111111111111111111111111111".to_string()));
    }

    #[test]
    fn test_extract_program_id_empty_instructions() {
        use crate::rpc::types::{TransactionData, Transaction, Message, TransactionMeta};
        
        let tx_data = TransactionData {
            transaction: Transaction {
                message: Message {
                    account_keys: vec!["SomeAccount111111111111111111111111111111".to_string()],
                    instructions: vec![],  // Empty!
                },
            },
            meta: None,
        };
        
        let program_id = extract_program_id(&tx_data);
        assert_eq!(program_id, None);
    }
}
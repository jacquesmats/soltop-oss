//! RPC client for interacting with Solana nodes
//!
//! This module provides functionality to fetch slots and blocks from any Solana RPC endpoint.

mod client;
mod types;
mod parser;

pub use client::RpcClient;
pub use types::{SlotResponse, BlockData, TransactionData, LogMessage};
pub use parser::{extract_cu, extract_program_id, extract_cu_timed};
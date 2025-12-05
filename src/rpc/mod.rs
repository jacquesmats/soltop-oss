//! RPC client for interacting with Solana nodes
//!
//! This module provides functionality to fetch slots and blocks from any Solana RPC endpoint.

mod client;
mod types;

pub use client::RpcClient;
pub use types::{SlotResponse, BlockData, TransactionData, LogMessage};
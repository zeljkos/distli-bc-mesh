// src/common/mod.rs - Updated to include smart contracts
pub mod blockchain;
pub mod types; 
pub mod crypto;
pub mod time;
pub mod api_utils;
pub mod contracts; // Add contracts module

// Re-export contract types for easy access
pub use contracts::{
    SmartContract, 
    ContractCall, 
    ContractResult, 
    ContractEvent, 
    ContractVM,
    create_trading_contract
};

// Re-export blockchain types with contract support
pub use blockchain::{
    Block,
    Transaction, 
    Blockchain
};

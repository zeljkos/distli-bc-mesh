// src/common/mod.rs - Updated to include smart contracts
pub mod blockchain;
pub mod types; 
pub mod crypto;
pub mod time;
pub mod api_utils;
pub mod contracts; // Add contracts module
pub mod zk_range_proofs; // Zero-knowledge range proofs with Bulletproofs
pub mod private_contracts; // Private contracts with ZK proofs
 // // Re-export commonly used types
pub use types::{
    Message,
    NetworkPeer,
    CrossNetworkTradeNotification  // <-- ADD THIS
};

// Re-export contract types for easy access
pub use contracts::{
    SmartContract, 
    ContractCall, 
    ContractResult, 
    ContractEvent, 
    ContractVM,
    create_trading_contract,
    create_gsm_roaming_contract
};

// Re-export private contract types
pub use private_contracts::{
    PrivateRoamingContract,
    PrivateContractManager,
    PrivateSession,
    PrivateSettlement,
    ContractTerms,
    ZKProof,
    RangeProof
};

// Re-export blockchain types with contract support
pub use blockchain::{
    Block,
    Transaction, 
    Blockchain
};

// Shared library for both tracker and enterprise blockchain
pub mod common;
pub mod tracker;
pub mod enterprise_bc;

// Re-export commonly used types
pub use common::{
    blockchain::{Block as TenantBlock, Transaction, Blockchain as TenantBlockchain},
    types::{Message, NetworkPeer},
    crypto::hash_data,
    time::current_timestamp,
};

// Enterprise blockchain exports - Fixed to use correct imports
pub use enterprise_bc::{
    EnterpriseBlock, 
    EnterpriseBlockchain,
    EnterpriseTransaction,
    TenantBlockchainUpdate,
    TenantBlockData,
    Validator,
    ConsensusEngine,
};

// Tracker exports  
pub use tracker::{
    Tracker,
    EnterpriseIntegration,
};

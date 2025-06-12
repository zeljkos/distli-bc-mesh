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

// Enterprise blockchain exports
pub use enterprise_bc::{
    blockchain::{EnterpriseBlock, TenantSummary, EnterpriseBlockchain},
    validator::Validator,
    consensus::ConsensusEngine,
};

// Tracker exports  
pub use tracker::{
    server::Tracker,
    integration::EnterpriseIntegration,
};

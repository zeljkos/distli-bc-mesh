// Enterprise blockchain module - master blockchain for tenant aggregation

// Use the actual uploaded filenames
pub mod blockchain; // Map blockchain_enteprise_bc.rs to blockchain module
pub mod validator;
pub mod consensus;
pub mod api;
pub mod dashboard;


// Re-export main types with shorter names
pub use blockchain::{EnterpriseBlock, TenantSummary, EnterpriseBlockchain};
pub use validator::Validator;
pub use consensus::ConsensusEngine;

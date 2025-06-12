// Enterprise blockchain module - master blockchain for tenant aggregation
pub mod blockchain;
pub mod validator;
pub mod consensus;
pub mod api;
pub mod dashboard;

// Re-export main types
pub use blockchain::{EnterpriseBlock, TenantSummary, EnterpriseBlockchain};
pub use validator::Validator;
pub use consensus::ConsensusEngine;

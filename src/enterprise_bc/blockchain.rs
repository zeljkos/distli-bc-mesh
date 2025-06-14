// Enterprise blockchain - aggregates tenant blockchain activities
use crate::common::{crypto::hash_data, time::current_timestamp, types::TenantUpdate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseBlock {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub tenant_summaries: Vec<TenantSummary>,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSummary {
    pub tenant_id: String,
    pub block_count: u64,
    pub transaction_count: u64,
    pub last_activity: u64,
    pub peer_count: usize,
    pub messages: Vec<String>, // Sample messages from tenant
}

#[derive(Debug, Clone)]
pub struct EnterpriseBlockchain {
    pub chain: Vec<EnterpriseBlock>,
    pub pending_tenant_updates: Vec<TenantUpdate>,
    pub validator_id: String,
    pub active_validators: std::collections::HashSet<String>, // ADD THIS
    pub last_validator_heartbeat: std::collections::HashMap<String, u64>, // ADD THIS
    pub storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseBlockchainData {
    pub chain: Vec<EnterpriseBlock>,
    pub pending_tenant_updates: Vec<TenantUpdate>,
    pub validator_id: String,
    pub active_validators: Vec<String>,
    pub last_validator_heartbeat: HashMap<String, u64>,
}

impl EnterpriseBlockchain {
    pub fn new(validator_id: String) -> Self {
        let storage_path = format!("data/enterprise_blockchain_{}.json", validator_id);
        let mut blockchain = EnterpriseBlockchain {
            chain: Vec::new(),
            pending_tenant_updates: Vec::new(),
            validator_id: validator_id.clone(),
            active_validators: std::collections::HashSet::new(),
            last_validator_heartbeat: std::collections::HashMap::new(),
            storage_path,
        };

        blockchain.load_from_disk();

        // If no data loaded, create genesis
        if blockchain.chain.is_empty() {
            let genesis = Self::create_genesis_block();
            blockchain.chain.push(genesis);
            blockchain.active_validators.insert(validator_id);
            blockchain.save_to_disk();
        }

        blockchain
    }
    pub fn save_to_disk(&self) {
        let data = EnterpriseBlockchainData {
            chain: self.chain.clone(),
            pending_tenant_updates: self.pending_tenant_updates.clone(),
            validator_id: self.validator_id.clone(),
            active_validators: self.active_validators.iter().cloned().collect(),
            last_validator_heartbeat: self.last_validator_heartbeat.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if let Some(parent) = Path::new(&self.storage_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub fn load_from_disk(&mut self) {
        if Path::new(&self.storage_path).exists() {
            if let Ok(content) = fs::read_to_string(&self.storage_path) {
                if let Ok(data) = serde_json::from_str::<EnterpriseBlockchainData>(&content) {
                    self.chain = data.chain;
                    self.pending_tenant_updates = data.pending_tenant_updates;
                    self.active_validators = data.active_validators.into_iter().collect();
                    self.last_validator_heartbeat = data.last_validator_heartbeat;
                }
            }
        }
    }
       // ADD THIS METHOD to track validator heartbeats
    pub fn update_validator_heartbeat(&mut self, validator_id: String) {
        use crate::common::time::current_timestamp;
        self.active_validators.insert(validator_id.clone());
        self.last_validator_heartbeat.insert(validator_id, current_timestamp());
        self.save_to_disk();
    }
       // ADD THIS METHOD to clean up stale validators
    pub fn cleanup_stale_validators(&mut self) {
        use crate::common::time::current_timestamp;
        let current_time = current_timestamp();
        let timeout = 120; // 2 minutes timeout

        let stale_validators: Vec<String> = self.last_validator_heartbeat
            .iter()
            .filter(|(_, &heartbeat)| current_time - heartbeat > timeout)
            .map(|(id, _)| id.clone())
            .collect();
        let has_stale_validators = !stale_validators.is_empty();

        for validator_id in stale_validators {
            self.active_validators.remove(&validator_id);
            self.last_validator_heartbeat.remove(&validator_id);
        }
        if has_stale_validators {
            self.save_to_disk();
        }
    }
    
    fn create_genesis_block() -> EnterpriseBlock {
        EnterpriseBlock {
            height: 0,
            hash: "0".repeat(64),
            previous_hash: "0".repeat(64),
            timestamp: current_timestamp(),
            validator: "genesis".to_string(),
            tenant_summaries: Vec::new(),
            merkle_root: "0".repeat(64),
        }
    }
    
    pub fn add_tenant_update(&mut self, update: TenantUpdate) {
        self.pending_tenant_updates.push(update);
        self.save_to_disk();
    }
    
    pub fn create_new_block(&mut self) -> EnterpriseBlock {
        let last_block = self.chain.last().unwrap();
        
        // Aggregate tenant updates into summaries
        let tenant_summaries = self.aggregate_tenant_updates();
        
        let mut new_block = EnterpriseBlock {
            height: last_block.height + 1,
            hash: String::new(),
            previous_hash: last_block.hash.clone(),
            timestamp: current_timestamp(),
            validator: self.validator_id.clone(),
            tenant_summaries: tenant_summaries.clone(),
            merkle_root: self.calculate_merkle_root(&tenant_summaries),
        };
        
        new_block.hash = self.calculate_block_hash(&new_block);
        new_block
    }
    
    pub fn add_block(&mut self, block: EnterpriseBlock) -> bool {
        if self.validate_block(&block) {
            self.chain.push(block);
            self.pending_tenant_updates.clear();
            self.save_to_disk();
            true
        } else {
            false
        }
    }
    
    fn aggregate_tenant_updates(&self) -> Vec<TenantSummary> {
        let mut summaries: HashMap<String, TenantSummary> = HashMap::new();
        
        for update in &self.pending_tenant_updates {
            let summary = summaries.entry(update.tenant_id.clone())
                .or_insert_with(|| TenantSummary {
                    tenant_id: update.tenant_id.clone(),
                    block_count: 0,
                    transaction_count: 0,
                    last_activity: 0,
                    peer_count: 0,
                    messages: Vec::new(),
                });
            
            summary.block_count += update.blocks_added;
            summary.transaction_count += update.transactions_added;
            summary.last_activity = update.timestamp.max(summary.last_activity);
            summary.peer_count = update.active_peers;
            summary.messages.extend(update.new_messages.clone());
            
            // Keep only last 5 messages per tenant
            if summary.messages.len() > 5 {
                summary.messages = summary.messages[summary.messages.len()-5..].to_vec();
            }
        }
        
        summaries.into_values().collect()
    }
    
    fn calculate_merkle_root(&self, summaries: &[TenantSummary]) -> String {
        if summaries.is_empty() {
            return "0".repeat(64);
        }
        
        let mut content = String::new();
        for summary in summaries {
            content.push_str(&serde_json::to_string(summary).unwrap_or_default());
        }
        hash_data(&content)
    }
    
    fn calculate_block_hash(&self, block: &EnterpriseBlock) -> String {
        let block_data = format!(
            "{}{}{}{}{}{}",
            block.height,
            block.previous_hash,
            block.timestamp,
            block.validator,
            block.merkle_root,
            serde_json::to_string(&block.tenant_summaries).unwrap_or_default()
        );
        hash_data(&block_data)
    }
    
    fn validate_block(&self, block: &EnterpriseBlock) -> bool {
        let last_block = self.chain.last().unwrap();
        
        // Check height
        if block.height != last_block.height + 1 {
            return false;
        }
        
        // Check previous hash
        if block.previous_hash != last_block.hash {
            return false;
        }
        
        // Verify hash
        if block.hash != self.calculate_block_hash(block) {
            return false;
        }
        
        true
    }
    
    pub fn get_latest_block(&self) -> &EnterpriseBlock {
        self.chain.last().unwrap()
    }
    
    pub fn get_blockchain_info(&self) -> serde_json::Value {
        let latest = self.get_latest_block();
        
        serde_json::json!({
            "height": latest.height,
            "latest_hash": latest.hash,
            "validator": self.validator_id,
            "pending_updates": self.pending_tenant_updates.len(),
            "total_blocks": self.chain.len(),
            "active_tenants": self.get_active_tenants().len(),
            "active_validators": self.active_validators.len() // ADD THIS LINE
        })
    }
    
    fn get_active_tenants(&self) -> Vec<String> {
        let mut tenants = std::collections::HashSet::new();
        if let Some(latest_block) = self.chain.last() {
            for summary in &latest_block.tenant_summaries {
                tenants.insert(summary.tenant_id.clone());
            }
        }
        tenants.into_iter().collect()
    }
}

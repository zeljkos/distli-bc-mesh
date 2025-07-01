// Updated enterprise blockchain - stores tenant blocks directly
use crate::common::{crypto::hash_data, time::current_timestamp};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseBlock {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub transactions: Vec<EnterpriseTransaction>,
    pub merkle_root: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseTransaction {
    pub tx_id: String,
    pub tenant_network: String,
    pub tenant_block_id: u64,
    pub tenant_block_hash: String,
    pub transaction_data: String,
    pub timestamp: u64,
    pub from_peer: String,
    pub contract_address: Option<String>,
    pub gas_used: Option<u64>,
    pub execution_result: Option<String>,
}

// Simple tenant block storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlock {
    pub network_id: String,
    pub block_id: u64,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub previous_hash: String,
    pub from_peer: String,
}

#[derive(Debug, Clone)]
pub struct EnterpriseBlockchain {
    pub chain: Vec<EnterpriseBlock>,
    pub pending_transactions: Vec<EnterpriseTransaction>,
    pub tenant_blocks: Vec<TenantBlock>, // NEW: Store tenant blocks directly
    pub validator_id: String,
    pub active_validators: std::collections::HashSet<String>,
    pub last_validator_heartbeat: std::collections::HashMap<String, u64>,
    pub storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockchainUpdate {
    pub network_id: String,
    pub peer_id: String,
    pub new_blocks: Vec<TenantBlockData>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockData {
    pub block_id: u64,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub previous_hash: String,
}

impl EnterpriseBlockchain {
    pub fn new(validator_id: String) -> Self {
        let storage_path = format!("data/enterprise_blockchain_{}.json", validator_id);
        let mut blockchain = EnterpriseBlockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            tenant_blocks: Vec::new(), // NEW
            validator_id: validator_id.clone(),
            active_validators: std::collections::HashSet::new(),
            last_validator_heartbeat: std::collections::HashMap::new(),
            storage_path,
        };

        blockchain.load_from_disk();

        if blockchain.chain.is_empty() {
            let genesis = Self::create_genesis_block();
            blockchain.chain.push(genesis);
            blockchain.active_validators.insert(validator_id);
            blockchain.save_to_disk();
        }

        blockchain
    }

    fn create_genesis_block() -> EnterpriseBlock {
        EnterpriseBlock {
            height: 0,
            hash: "0".repeat(64),
            previous_hash: "0".repeat(64),
            timestamp: current_timestamp(),
            validator: "genesis".to_string(),
            transactions: Vec::new(),
            merkle_root: "0".repeat(64),
            nonce: 0,
        }
    }

    // NEW: Store tenant block directly (like full blockchain copy)
    pub fn add_tenant_block_directly(
        &mut self,
        network_id: &str,
        block_id: u64,
        block_hash: &str,
        transactions: &[String],
        timestamp: u64,
        previous_hash: &str,
        from_peer: &str
    ) {
        // Check if we already have this block
        let exists = self.tenant_blocks.iter().any(|b| 
            b.network_id == network_id && b.block_id == block_id
        );
        
        if !exists {
            let tenant_block = TenantBlock {
                network_id: network_id.to_string(),
                block_id,
                block_hash: block_hash.to_string(),
                transactions: transactions.to_vec(),
                timestamp,
                previous_hash: previous_hash.to_string(),
                from_peer: from_peer.to_string(),
            };
            
            self.tenant_blocks.push(tenant_block);
            self.save_to_disk();
        }
    }

    // NEW: Get recent tenant blocks for display
    pub fn get_recent_tenant_blocks(&self, limit: usize) -> Vec<serde_json::Value> {
        let start_idx = if self.tenant_blocks.len() > limit {
            self.tenant_blocks.len() - limit
        } else {
            0
        };

        self.tenant_blocks[start_idx..].iter().map(|block| {
            serde_json::json!({
                "network_id": block.network_id,
                "block_id": block.block_id,
                "block_hash": block.block_hash,
                "transactions": block.transactions,
                "timestamp": block.timestamp,
                "previous_hash": block.previous_hash,
                "from_peer": block.from_peer
            })
        }).collect()
    }

    // NEW: Get tenant summaries from stored blocks
    pub fn get_tenant_summaries(&self) -> Vec<serde_json::Value> {
        let mut tenant_stats: std::collections::HashMap<String, (usize, usize, u64, Vec<String>)> = std::collections::HashMap::new();
        
        for block in &self.tenant_blocks {
            let entry = tenant_stats.entry(block.network_id.clone())
                .or_insert((0, 0, 0, Vec::new()));
            
            entry.0 += 1; // block count
            entry.1 += block.transactions.len(); // transaction count
            entry.2 = entry.2.max(block.timestamp); // last activity
            
            // Add recent messages
            for tx in &block.transactions {
                entry.3.push(format!("Block #{}: {}", block.block_id, tx));
                if entry.3.len() > 3 {
                    entry.3.remove(0);
                }
            }
        }
        
        tenant_stats.into_iter().map(|(network_id, (blocks, txs, last_activity, messages))| {
            serde_json::json!({
                "tenant_id": network_id,
                "block_count": blocks,
                "transaction_count": txs,
                "last_activity": last_activity,
                "recent_messages": messages
            })
        }).collect()
    }

    // Keep existing methods for backward compatibility
    pub fn add_tenant_transactions(&mut self, update: TenantBlockchainUpdate) {
        for block in update.new_blocks {
            for (tx_index, tx_data) in block.transactions.iter().enumerate() {
                let enterprise_tx = EnterpriseTransaction {
                    tx_id: format!("{}_{}_{}_{}", update.network_id, block.block_id, tx_index, current_timestamp()),
                    tenant_network: update.network_id.clone(),
                    tenant_block_id: block.block_id,
                    tenant_block_hash: block.block_hash.clone(),
                    transaction_data: tx_data.clone(),
                    timestamp: block.timestamp,
                    from_peer: update.peer_id.clone(),
                    contract_address: None,
                    gas_used: None,
                    execution_result: None,
                };
                
                self.pending_transactions.push(enterprise_tx);
            }
        }
        self.save_to_disk();
    }

    pub fn mine_block(&mut self) -> Option<EnterpriseBlock> {
        if self.pending_transactions.is_empty() {
            return None;
        }

        let last_block = match self.chain.last() {
            Some(block) => block,
            None => return None,
        };
        
        let transactions = self.pending_transactions.clone();
        
        let mut new_block = EnterpriseBlock {
            height: last_block.height + 1,
            hash: String::new(),
            previous_hash: last_block.hash.clone(),
            timestamp: current_timestamp(),
            validator: self.validator_id.clone(),
            transactions: transactions.clone(),
            merkle_root: self.calculate_merkle_root(&transactions),
            nonce: 0,
        };

        self.mine_block_pow(&mut new_block);
        Some(new_block)
    }

    fn mine_block_pow(&self, block: &mut EnterpriseBlock) {
        loop {
            block.hash = self.calculate_block_hash(block);
            if block.hash.starts_with("00") {
                break;
            }
            block.nonce += 1;
        }
    }

    pub fn add_block(&mut self, block: EnterpriseBlock) -> bool {
        if self.validate_block(&block) {
            self.chain.push(block);
            self.pending_transactions.clear();
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    fn calculate_merkle_root(&self, transactions: &[EnterpriseTransaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }
        
        let tx_hashes: Vec<String> = transactions.iter()
            .map(|tx| hash_data(&format!("{}{}{}", tx.tx_id, tx.transaction_data, tx.timestamp)))
            .collect();
        
        hash_data(&tx_hashes.join(""))
    }

    fn calculate_block_hash(&self, block: &EnterpriseBlock) -> String {
        let block_data = format!(
            "{}{}{}{}{}{}",
            block.height,
            block.previous_hash,
            block.timestamp,
            block.validator,
            block.merkle_root,
            block.nonce
        );
        hash_data(&block_data)
    }

    fn validate_block(&self, block: &EnterpriseBlock) -> bool {
        let last_block = match self.chain.last() {
            Some(block) => block,
            None => return false,
        };
        
        if block.height != last_block.height + 1 {
            return false;
        }
        
        if block.previous_hash != last_block.hash {
            return false;
        }
        
        if block.hash != self.calculate_block_hash(block) {
            return false;
        }

        if !block.hash.starts_with("00") {
            return false;
        }
        
        true
    }

    pub fn get_blockchain_info(&self) -> serde_json::Value {
        let total_transactions: usize = self.tenant_blocks.iter()
            .map(|block| block.transactions.len())
            .sum();
        
        let tenant_networks: std::collections::HashSet<String> = self.tenant_blocks.iter()
            .map(|block| block.network_id.clone())
            .collect();
        
        serde_json::json!({
            "height": self.tenant_blocks.len(),
            "validator": self.validator_id,
            "total_blocks": self.tenant_blocks.len(),
            "total_transactions": total_transactions,
            "active_validators": 1,
            "active_tenants": tenant_networks.len(),
            "chain_health": "healthy",
            "validator_status": "online"
        })
    }

    pub fn save_to_disk(&self) {
        let data = serde_json::json!({
            "chain": self.chain,
            "pending_transactions": self.pending_transactions,
            "tenant_blocks": self.tenant_blocks, // NEW
            "validator_id": self.validator_id,
            "active_validators": self.active_validators.iter().cloned().collect::<Vec<_>>(),
            "last_validator_heartbeat": self.last_validator_heartbeat,
        });

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
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Ok(chain) = serde_json::from_value(data["chain"].clone()) {
                        self.chain = chain;
                    }
                    if let Ok(pending) = serde_json::from_value(data["pending_transactions"].clone()) {
                        self.pending_transactions = pending;
                    }
                    if let Ok(tenant_blocks) = serde_json::from_value(data["tenant_blocks"].clone()) {
                        self.tenant_blocks = tenant_blocks; // NEW
                    }
                    if let Ok(validators) = serde_json::from_value::<Vec<String>>(data["active_validators"].clone()) {
                        self.active_validators = validators.into_iter().collect();
                    }
                    if let Ok(heartbeat) = serde_json::from_value(data["last_validator_heartbeat"].clone()) {
                        self.last_validator_heartbeat = heartbeat;
                    }
                }
            }
        }
    }

    pub fn update_validator_heartbeat(&mut self, validator_id: String) {
        self.active_validators.insert(validator_id.clone());
        self.last_validator_heartbeat.insert(validator_id, current_timestamp());
        self.save_to_disk();
    }

    pub fn cleanup_stale_validators(&mut self) {
        let current_time = current_timestamp();
        let timeout = 120;

        let stale_validators: Vec<String> = self.last_validator_heartbeat
            .iter()
            .filter(|(_, &heartbeat)| current_time - heartbeat > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for validator_id in stale_validators {
            self.active_validators.remove(&validator_id);
            self.last_validator_heartbeat.remove(&validator_id);
        }
    }
}

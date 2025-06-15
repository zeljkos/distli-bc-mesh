// Updated integration.rs - sends full transaction data, not summaries
use crate::common::time::current_timestamp;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub type Networks = std::sync::Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, crate::common::types::NetworkPeer>>>>;

// NEW: Full blockchain update with actual transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainUpdate {
    pub network_id: String,
    pub peer_id: String,
    pub new_blocks: Vec<BlockData>, // CHANGED: Send actual blocks, not just counts
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub id: u64,
    pub hash: String,
    pub data: String,           // Full transaction content
    pub timestamp: u64,
    pub previous_hash: String,
    pub nonce: u64,
}

pub struct EnterpriseIntegration {
    enterprise_url: String,
    last_reported_state: HashMap<String, TenantState>,
    network_blockchain_state: HashMap<String, NetworkBlockchainState>,
    storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TenantState {
    last_reported_block_id: u64,
    last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkBlockchainState {
    blocks: Vec<BlockData>,  // Store full block data
    last_block_id: u64,
    last_update: u64,
}

impl EnterpriseIntegration {
    pub fn new(enterprise_url: String) -> Self {
        let storage_path = "data/tracker_integration.json".to_string();
        let mut integration = EnterpriseIntegration {
            enterprise_url,
            last_reported_state: HashMap::new(),
            network_blockchain_state: HashMap::new(),
            storage_path,
        };
        integration.load_from_disk();
        integration
    }
    
    pub async fn start_reporting_loop(&mut self, networks: Networks) {
        info!("Starting enterprise blockchain reporting to: {}", self.enterprise_url);
        
        let mut timer = interval(Duration::from_secs(10)); // Check every 10 seconds
        
        loop {
            timer.tick().await;
            
            if let Err(e) = self.check_and_report_new_blocks(&networks).await {
                warn!("Failed to report to enterprise blockchain: {}", e);
            }
        }
    }
    
    // NEW: Send full transaction data when new blocks arrive
    pub async fn update_network_blockchain_state_with_blocks(&mut self, update: &BlockchainUpdate) {
        info!("Updating blockchain state for {}: {} new blocks", 
              update.network_id, update.new_blocks.len());
        
        let state = self.network_blockchain_state.entry(update.network_id.clone())
            .or_insert_with(|| NetworkBlockchainState {
                blocks: Vec::new(),
                last_block_id: 0,
                last_update: current_timestamp(),
            });

        let mut new_blocks = Vec::new();
        
        // Add only truly new blocks
        for block in &update.new_blocks {
            if block.id > state.last_block_id {
                state.blocks.push(block.clone());
                state.last_block_id = block.id;
                new_blocks.push(block.clone());
                info!("Added new block #{} to {}: \"{}\"", block.id, update.network_id, block.data);
            }
        }
        
        state.last_update = current_timestamp();
        
        // Keep only the last 100 blocks to prevent memory bloat
        if state.blocks.len() > 100 {
            state.blocks = state.blocks[state.blocks.len()-100..].to_vec();
        }
        
        self.save_to_disk();
        
        // IMMEDIATELY send new blocks to enterprise BC
        //if !new_blocks.is_empty() {
         //   self.send_blocks_to_enterprise(&update.network_id, &new_blocks).await;
       // }
        if let Err(e) = self.send_blocks_to_enterprise(&update.network_id, &new_blocks).await {
            warn!("Failed to send blocks to enterprise BC: {}", e);
        }
    }
    
    // NEW: Send actual blocks to enterprise blockchain immediately
    async fn send_blocks_to_enterprise(&mut self, network_id: &str, blocks: &[BlockData]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Sending {} blocks from {} to enterprise BC", blocks.len(), network_id);
        
        // Convert to enterprise format
        let enterprise_blocks: Vec<serde_json::Value> = blocks.iter().map(|block| {
            let transactions = if block.data == "Genesis Block" {
                vec![]
            } else {
                // Split comma-separated transactions or treat as single transaction
                if block.data.contains(',') {
                    block.data.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![block.data.clone()]
                }
            };
            
            serde_json::json!({
                "block_id": block.id,
                "block_hash": block.hash,
                "transactions": transactions,
                "timestamp": block.timestamp,
                "previous_hash": block.previous_hash
            })
        }).collect();
        
        let update = serde_json::json!({
            "network_id": network_id,
            "peer_id": "tracker",
            "new_blocks": enterprise_blocks,
            "timestamp": current_timestamp()
        });
        
        let client = reqwest::Client::new();
        let url = format!("{}/api/tenant-blockchain-update", self.enterprise_url);
        
        let response = client
            .post(&url)
            .json(&update)
            .send()
            .await?;
            
        if response.status().is_success() {
            info!("✅ Successfully sent {} blocks from {} to enterprise BC", blocks.len(), network_id);
            
            // Update reported state
            self.last_reported_state.insert(network_id.to_string(), TenantState {
                last_reported_block_id: blocks.last().unwrap().id,
                last_update: current_timestamp(),
            });
            self.save_to_disk();
            
            // Log what was sent
            for block in blocks {
                if block.data != "Genesis Block" {
                    info!("📤 Sent to enterprise: Block #{}: \"{}\"", block.id, block.data);
                }
            }
        } else {
            warn!("Failed to send blocks to enterprise BC: {}", response.status());
        }
        
        Ok(())
    }
    
    // Check for new blocks and report them
    async fn check_and_report_new_blocks(&mut self, networks: &Networks) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (network_id, _peers) in networks.read().await.iter() {
            if let Some(state) = self.network_blockchain_state.get(network_id) {
                let last_reported = self.last_reported_state.get(network_id)
                    .map(|s| s.last_reported_block_id)
                    .unwrap_or(0);
                
                // Find unreported blocks
                let unreported_blocks: Vec<BlockData> = state.blocks.iter()
                    .filter(|block| block.id > last_reported)
                    .cloned()
                    .collect();
                
                if !unreported_blocks.is_empty() {
                    info!("Found {} unreported blocks in {}", unreported_blocks.len(), network_id);
                    self.send_blocks_to_enterprise(network_id, &unreported_blocks).await?;
                }
            }
        }
        
        Ok(())
    }
    
    // Persistence methods
    pub fn save_to_disk(&self) {
        let data = serde_json::json!({
            "last_reported_state": self.last_reported_state,
            "network_blockchain_state": self.network_blockchain_state,
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
                    if let Ok(reported) = serde_json::from_value(data["last_reported_state"].clone()) {
                        self.last_reported_state = reported;
                    }
                    if let Ok(state) = serde_json::from_value(data["network_blockchain_state"].clone()) {
                        self.network_blockchain_state = state;
                    }
                }
            }
        }
    }
}

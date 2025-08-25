// Fixed integration.rs - uses enterprise types throughout
use crate::blockchain::{TenantBlockchainUpdate, TenantBlockData};
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// Simple time utility
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub type Networks = std::sync::Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, crate::tracker::server::NetworkPeer>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TenantState {
    last_reported_block_id: u32,  // Changed from u64 to u32
    last_update: u64,  // Keep u64 for timestamps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkBlockchainState {
    blocks: Vec<TenantBlockData>,
    last_block_id: u32,  // Changed from u64 to u32
    last_update: u64,  // Keep u64 for timestamps
}

pub struct EnterpriseIntegration {
    enterprise_url: String,
    last_reported_state: HashMap<String, TenantState>,
    network_blockchain_state: HashMap<String, NetworkBlockchainState>,
    storage_path: String,
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
        
        let mut timer = interval(Duration::from_secs(10));
        
        loop {
            timer.tick().await;
            
            if let Err(e) = self.check_and_report_new_blocks(&networks).await {
                warn!("Failed to report to enterprise blockchain: {}", e);
            }
        }
    }
    
    // Accept enterprise format directly
    pub async fn update_network_blockchain_state_with_update(&mut self, update: &TenantBlockchainUpdate) {
        info!("Updating blockchain state for {}: {} new blocks", 
              update.network_id, update.new_blocks.len());
        
        if update.new_blocks.is_empty() {
            warn!("Received empty blocks list for network {}", update.network_id);
            return;
        }
        
        let state = self.network_blockchain_state.entry(update.network_id.clone())
            .or_insert_with(|| NetworkBlockchainState {
                blocks: Vec::new(),
                last_block_id: 0,
                last_update: current_timestamp(),
            });

        let mut new_blocks = Vec::new();
        
        for block in &update.new_blocks {
            if block.block_id > state.last_block_id {
                state.blocks.push(block.clone());
                state.last_block_id = block.block_id;
                new_blocks.push(block.clone());
                info!("Added new block #{} to {}", block.block_id, update.network_id);
            }
        }
        
        state.last_update = current_timestamp();
        
        // Keep only the last 100 blocks
        if state.blocks.len() > 100 {
            let start_idx = state.blocks.len() - 100;
            state.blocks = state.blocks[start_idx..].to_vec();
        }
        
        self.save_to_disk();
        
        // Send to enterprise BC
        if !new_blocks.is_empty() {
            if let Err(e) = self.send_update_to_enterprise(update).await {
                warn!("Failed to send update to enterprise BC: {}", e);
            }
        } else {
            info!("No new blocks to send to enterprise BC for {}", update.network_id);
        }
    }
    
    // Send the update as-is to enterprise BC
    async fn send_update_to_enterprise(&mut self, update: &TenantBlockchainUpdate) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Sending update from {} to enterprise BC", update.network_id);
        
        if update.new_blocks.is_empty() {
            info!("No blocks to send for network {}", update.network_id);
            return Ok(());
        }
        
        let client = reqwest::Client::new();
        let url = format!("{}/api/tenant-blockchain-update", self.enterprise_url);
        
        let response = client
            .post(&url)
            .json(update)
            .send()
            .await?;
            
        if response.status().is_success() {
            info!("✅ Successfully sent update from {} to enterprise BC", update.network_id);
            
            // Safe access to last block
            if let Some(last_block) = update.new_blocks.last() {
                self.last_reported_state.insert(update.network_id.clone(), TenantState {
                    last_reported_block_id: last_block.block_id,
                    last_update: current_timestamp(),
                });
                self.save_to_disk();
                info!("Updated last reported block to {} for network {}", 
                      last_block.block_id, update.network_id);
            } else {
                warn!("No blocks in update for network {}", update.network_id);
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            warn!("Failed to send update to enterprise BC: {} - {}", status, error_text);
        }
        
        Ok(())
    }
    
    async fn check_and_report_new_blocks(&mut self, _networks: &Networks) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network_ids: Vec<String> = self.network_blockchain_state.keys().cloned().collect();
        
        for network_id in network_ids {
            let last_reported = self.last_reported_state.get(&network_id)
                .map(|s| s.last_reported_block_id)
                .unwrap_or(0);
            
            let unreported_blocks: Vec<TenantBlockData> = self.network_blockchain_state
                .get(&network_id)
                .map(|state| {
                    state.blocks.iter()
                        .filter(|block| block.block_id > last_reported)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            
            if !unreported_blocks.is_empty() {
                info!("Found {} unreported blocks in {}", unreported_blocks.len(), network_id);
                
                let update = TenantBlockchainUpdate {
                    network_id: network_id.clone(),
                    peer_id: "tracker".to_string(),
                    new_blocks: unreported_blocks,
                    timestamp: current_timestamp(),
                };
                
                if let Err(e) = self.send_update_to_enterprise(&update).await {
                    warn!("Failed to send unreported blocks for {}: {}", network_id, e);
                }
            }
        }
        
        Ok(())
    }
    
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

// Enterprise blockchain integration for tracker
use crate::common::types::{TenantUpdate, NetworkStats};
use crate::common::time::current_timestamp;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

// REMOVE this import as Networks will be passed as parameter
// use crate::tracker::server::Networks;

// Define Networks type locally to avoid circular imports
pub type Networks = std::sync::Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, crate::common::types::NetworkPeer>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainUpdate {
    pub network_id: String,
    pub peer_id: String,
    pub block_count: u64,
    pub transaction_count: u64,
    pub latest_block_hash: String,
    pub timestamp: u64,
    // ADD: Include actual block data
    pub latest_blocks: Vec<BlockData>, // NEW: Actual block content
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub id: u64,
    pub hash: String,
    pub data: String,
    pub timestamp: u64,
}

pub struct EnterpriseIntegration {
    enterprise_url: String,
    last_reported_state: HashMap<String, TenantState>,
    network_blockchain_state: HashMap<String, NetworkBlockchainState>,
}

#[derive(Debug, Clone)]
struct TenantState {
    block_count: u64,
    transaction_count: u64,
    last_update: u64,
    last_reported_block_id: u64, // NEW: Track last reported block
}

#[derive(Debug, Clone)]
struct NetworkBlockchainState {
    total_blocks: u64,
    total_transactions: u64,
    last_update: u64,
    recent_blocks: Vec<BlockData>, // NEW: Store actual block data
    last_block_id: u64, // NEW: Track highest block ID seen
}

impl EnterpriseIntegration {
    pub fn new(enterprise_url: String) -> Self {
        EnterpriseIntegration {
            enterprise_url,
            last_reported_state: HashMap::new(),
            network_blockchain_state: HashMap::new(),
        }
    }
    
    pub async fn start_reporting_loop(&mut self, networks: Networks) {
        info!("Starting enterprise blockchain reporting to: {}", self.enterprise_url);
        
        let mut timer = interval(Duration::from_secs(5)); // Report every minute
        
        loop {
            timer.tick().await;
            
            if let Err(e) = self.collect_and_report_tenant_updates(&networks).await {
                warn!("Failed to report to enterprise blockchain: {}", e);
            }
        }
    }
     pub async fn report_immediately(&mut self, network_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(state) = self.network_blockchain_state.get(network_id) {
            println!("📋 Immediate report for {}: {} total blocks in state", network_id, state.total_blocks);

            // Generate messages from actual stored blocks
            let messages = self.generate_block_messages(&state.recent_blocks);
            println!("📝 Generated immediate messages: {:?}", messages);

            let last_state = self.last_reported_state.get(network_id);

            let blocks_added = match last_state {
                Some(s) => state.total_blocks.saturating_sub(s.block_count),
                None => state.total_blocks,
            };

            let transactions_added = match last_state {
                Some(s) => state.total_transactions.saturating_sub(s.transaction_count),
                None => state.total_transactions,
            };

            if blocks_added > 0 || transactions_added > 0 || last_state.is_none() {
                let update = TenantUpdate {
                    tenant_id: network_id.to_string(),
                    blocks_added,
                    transactions_added,
                    new_messages: messages.clone(),
                    active_peers: 1,
                    timestamp: current_timestamp(),
                };

                println!("🚀 Sending IMMEDIATE update with messages: {:?}", messages);
                self.send_update_to_enterprise(&update).await?;

                // Update tracking state
                self.last_reported_state.insert(network_id.to_string(), TenantState {
                    block_count: state.total_blocks,
                    transaction_count: state.total_transactions,
                    last_update: update.timestamp,
                    last_reported_block_id: state.last_block_id,
                });

                println!("✅ Immediate report completed for {}", network_id);
            } else {
                println!("ℹ️ No changes to report for {}", network_id);
            }
        } else {
            println!("⚠️ No blockchain state found for {}", network_id);
        }
        Ok(())
    }
    
    async fn collect_and_report_tenant_updates(
        &mut self,
        networks: &Networks
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let networks_lock = networks.read().await;
        
        for (network_id, peers) in networks_lock.iter() {
            let current_peer_count = peers.len();
            
            // GET REAL BLOCKCHAIN DATA from stored state
            let (current_blocks, current_transactions, new_block_messages) = if let Some(state) = self.network_blockchain_state.get(network_id) {
                let messages = self.generate_block_messages(&state.recent_blocks);
                (state.total_blocks, state.total_transactions, messages)
            } else {
                // Fallback to peer count if no blockchain data yet
                (current_peer_count as u64, current_peer_count as u64, Vec::new())
            };
            
            let last_state = self.last_reported_state.get(network_id);
            
            let blocks_added = match last_state {
                Some(state) => current_blocks.saturating_sub(state.block_count),
                None => current_blocks,
            };
            
            let transactions_added = match last_state {
                Some(state) => current_transactions.saturating_sub(state.transaction_count),
                None => current_transactions,
            };
            
            // Only report if there are changes
            if blocks_added > 0 || transactions_added > 0 || last_state.is_none() {
                // Use actual block content for messages
                let messages = if new_block_messages.is_empty() {
                    self.generate_real_messages(network_id, blocks_added, transactions_added)
                } else {
                    new_block_messages
                };
                
                let update = TenantUpdate {
                    tenant_id: network_id.to_string(), // FIX: Convert &str to String
                    blocks_added,
                    transactions_added,
                    new_messages: messages,
                    active_peers: current_peer_count,
                    timestamp: current_timestamp(),
                };
                
                self.send_update_to_enterprise(&update).await?;
                
                // Update our tracking state - FIX: Convert &str to String
                self.last_reported_state.insert(network_id.to_string(), TenantState {
                    block_count: current_blocks,
                    transaction_count: current_transactions,
                    last_update: update.timestamp,
                    last_reported_block_id: current_blocks.saturating_sub(1), // Track last reported block
                });
                
                info!("Reported REAL update for tenant {}: {} blocks, {} transactions, {} peers", 
                      network_id, blocks_added, transactions_added, current_peer_count);
            }
        }
        
        Ok(())
    }
    
    async fn send_update_to_enterprise(
        &self,
        update: &TenantUpdate
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tenant-update", self.enterprise_url);
        
        let response = client
            .post(&url)
            .json(update)
            .send()
            .await?;
            
        if response.status().is_success() {
            info!("Successfully sent update for tenant: {}", update.tenant_id);
        } else {
            warn!("Failed to send update for tenant {}: {}", 
                  update.tenant_id, response.status());
        }
        
        Ok(())
    }
    
    // NEW: Generate messages from actual block content
    fn generate_block_messages(&self, blocks: &[BlockData]) -> Vec<String> {
        let mut messages = Vec::new();
        
        // Get the 3 most recent blocks
        let recent_blocks: Vec<&BlockData> = blocks.iter().rev().take(3).collect();
        
        for block in recent_blocks {
            if block.data != "Genesis Block" {
                messages.push(format!("Block #{}: \"{}\"", block.id, block.data));
            }
        }
        
        if messages.is_empty() {
            messages.push("New blockchain activity".to_string());
        }
        
        messages
    }
    
    // Generate real messages based on blockchain activity
    fn generate_real_messages(&self, network_id: &str, blocks_added: u64, transactions_added: u64) -> Vec<String> {
        let mut messages = Vec::new();
        
        if blocks_added > 0 {
            messages.push(format!("New block mined in {}", network_id));
            if blocks_added > 1 {
                messages.push(format!("{} blocks added to {}", blocks_added, network_id));
            }
        }
        if transactions_added > 0 {
            messages.push(format!("{} transactions processed in {}", transactions_added, network_id));
        }
        if blocks_added == 0 && transactions_added == 0 {
            messages.push(format!("Network {} is active with peers", network_id));
        }
        
        // Return last 3 messages
        messages.into_iter().take(3).collect()
    }
    
    // ENHANCED: Method to update blockchain state with actual block data
    pub fn update_network_blockchain_state_with_blocks(&mut self, update: &BlockchainUpdate) {
        info!("Updating blockchain state for {}: {} blocks, {} transactions", 
              update.network_id, update.block_count, update.transaction_count);
        
        let state = self.network_blockchain_state.entry(update.network_id.clone())
            .or_insert_with(|| NetworkBlockchainState {
                total_blocks: 0,
                total_transactions: 0,
                last_update: current_timestamp(),
                recent_blocks: Vec::new(),
                last_block_id: 0,
            });

        // Update with the latest values
        state.total_blocks = update.block_count.max(state.total_blocks);
        state.total_transactions = update.transaction_count.max(state.total_transactions);
        state.last_update = current_timestamp();
        
        // Add new blocks to recent_blocks
        for block in &update.latest_blocks {
            if block.id > state.last_block_id {
                state.recent_blocks.push(block.clone());
                state.last_block_id = block.id;
                info!("Added new block #{} to {}: \"{}\"", block.id, update.network_id, block.data);
            }
        }
        
        // Keep only the last 10 blocks
        if state.recent_blocks.len() > 10 {
            state.recent_blocks = state.recent_blocks[state.recent_blocks.len()-10..].to_vec();
        }
        
        info!("Updated state for {}: {} blocks, {} transactions, {} recent blocks", 
              update.network_id, state.total_blocks, state.total_transactions, state.recent_blocks.len());
    }
    
    // BACKWARD COMPATIBILITY: Keep the old method for simple updates
    pub fn update_network_blockchain_state(&mut self, network_id: String, blocks: u64, transactions: u64) {
        let simple_update = BlockchainUpdate {
            network_id,
            peer_id: "legacy".to_string(),
            block_count: blocks,
            transaction_count: transactions,
            latest_block_hash: "unknown".to_string(),
            timestamp: current_timestamp(),
            latest_blocks: Vec::new(),
        };
        
        self.update_network_blockchain_state_with_blocks(&simple_update);
    }
    
    pub async fn get_network_stats(&self, networks: &Networks) -> Vec<NetworkStats> {
        let networks_lock = networks.read().await;
        let mut stats = Vec::new();
        
        for (network_id, peers) in networks_lock.iter() {
            let peer_count = peers.len();
            // FIX: Specify generic type parameter for get method
            let (block_count, transaction_count) = match self.network_blockchain_state.get(network_id as &str) {
                Some(state) => (state.total_blocks, state.total_transactions),
                None => match self.last_reported_state.get(network_id as &str) {
                    Some(state) => (state.block_count, state.transaction_count),
                    None => (0, 0),
                }
            };
            
            stats.push(NetworkStats {
                network_id: network_id.to_string(), // FIX: Convert &str to String
                peer_count,
                block_count,
                transaction_count,
                last_activity: current_timestamp(),
            });
        }
        
        stats
    }
}

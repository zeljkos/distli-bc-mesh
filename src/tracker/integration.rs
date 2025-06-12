// Enterprise blockchain integration for tracker
use crate::common::types::{TenantUpdate, NetworkStats};
use crate::common::time::current_timestamp;
use crate::tracker::server::Networks;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct EnterpriseIntegration {
    enterprise_url: String,
    last_reported_state: HashMap<String, TenantState>,
}

#[derive(Debug, Clone)]
struct TenantState {
    block_count: u64,
    transaction_count: u64,
    last_update: u64,
}

impl EnterpriseIntegration {
    pub fn new(enterprise_url: String) -> Self {
        EnterpriseIntegration {
            enterprise_url,
            last_reported_state: HashMap::new(),
        }
    }
    
    pub async fn start_reporting_loop(&mut self, networks: Networks) {
        info!("Starting enterprise blockchain reporting to: {}", self.enterprise_url);
        
        let mut timer = interval(Duration::from_secs(60)); // Report every minute
        
        loop {
            timer.tick().await;
            
            if let Err(e) = self.collect_and_report_tenant_updates(&networks).await {
                warn!("Failed to report to enterprise blockchain: {}", e);
            }
        }
    }
    
    async fn collect_and_report_tenant_updates(
        &mut self,
        networks: &Networks
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let networks_lock = networks.read().await;
        
        for (network_id, peers) in networks_lock.iter() {
            let current_peer_count = peers.len();
            
            // Simulate blockchain state (in real implementation, 
            // you'd get this from actual tenant blockchain data)
            let current_blocks = (current_peer_count as u64) * 10; // Simulate blocks
            let current_transactions = current_blocks * 5; // Simulate transactions
            
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
                let sample_messages = self.generate_sample_messages(network_id, current_peer_count);
                
                let update = TenantUpdate {
                    tenant_id: network_id.clone(),
                    blocks_added,
                    transactions_added,
                    new_messages: sample_messages,
                    active_peers: current_peer_count,
                    timestamp: current_timestamp(),
                };
                
                self.send_update_to_enterprise(&update).await?;
                
                // Update our tracking state
                self.last_reported_state.insert(network_id.clone(), TenantState {
                    block_count: current_blocks,
                    transaction_count: current_transactions,
                    last_update: update.timestamp,
                });
                
                info!("Reported update for tenant {}: {} blocks, {} transactions, {} peers", 
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
    
    fn generate_sample_messages(&self, network_id: &str, peer_count: usize) -> Vec<String> {
        if peer_count == 0 {
            return Vec::new();
        }
        
        // Generate some sample messages based on network activity
        let messages = vec![
            format!("New peer joined {}", network_id),
            format!("Transaction processed in {}", network_id),
            format!("Block mined by {}", network_id),
            format!("{} peers active in {}", peer_count, network_id),
        ];
        
        // Return last 2 messages
        messages.into_iter().take(2).collect()
    }
    
    pub async fn get_network_stats(&self, networks: &Networks) -> Vec<NetworkStats> {
        let networks_lock = networks.read().await;
        let mut stats = Vec::new();
        
        for (network_id, peers) in networks_lock.iter() {
            let peer_count = peers.len();
            let (block_count, transaction_count) = match self.last_reported_state.get(network_id) {
                Some(state) => (state.block_count, state.transaction_count),
                None => (0, 0),
            };
            
            stats.push(NetworkStats {
                network_id: network_id.clone(),
                peer_count,
                block_count,
                transaction_count,
                last_activity: current_timestamp(),
            });
        }
        
        stats
    }
}

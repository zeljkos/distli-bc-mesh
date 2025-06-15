// Updated validator.rs - creates block for every tenant transaction
use crate::enterprise_bc::{EnterpriseBlockchain, TenantBlockchainUpdate};
use crate::enterprise_bc::api;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct Validator {
    pub id: String,
    pub port: u16,
    pub blockchain: Arc<RwLock<EnterpriseBlockchain>>,
    pub peers: Vec<String>,
}

impl Validator {
    pub async fn new(id: String, port: u16, peers: Vec<String>) -> Self {
        let blockchain = Arc::new(RwLock::new(EnterpriseBlockchain::new(id.clone())));
        
        Validator {
            id,
            port,
            blockchain,
            peers,
        }
    }
    
    pub async fn start(self) {
        info!("Starting validator {} on port {}", self.id, self.port);
        
        let blockchain = self.blockchain.clone();
        let validator_id = self.id.clone();
        
        // Start heartbeat timer (keep validator alive)
        let heartbeat_blockchain = blockchain.clone();
        let heartbeat_validator_id = validator_id.clone();
        let heartbeat_handle = tokio::spawn(async move {
            Self::heartbeat_loop(heartbeat_blockchain, heartbeat_validator_id).await;
        });
        
        // Start API server
        let api_blockchain = blockchain.clone();
        let api_handle = tokio::spawn(async move {
            api::start_api_server(self.port, api_blockchain).await;
        });
        
        info!("Validator {} fully started - ready for immediate block creation", self.id);
        
        // Wait for all services
        tokio::select! {
            _ = heartbeat_handle => warn!("Heartbeat stopped"),
            _ = api_handle => warn!("API server stopped"),
        }
    }
    
    // NEW: Immediate block creation when tenant transactions arrive
    pub async fn process_tenant_update(&self, update: TenantBlockchainUpdate) -> bool {
        info!("Processing tenant update from network: {}", update.network_id);
        
        let new_block = {
            let mut bc = self.blockchain.write().await;
            
            // Add tenant transactions to pending pool
            bc.add_tenant_transactions(update);
            
            // IMMEDIATELY mine a block if there are pending transactions
            if let Some(block) = bc.mine_block() {
                info!("Mined block {} with {} transactions", 
                      block.height, block.transactions.len());
                
                // Log actual transaction content
                for tx in &block.transactions {
                    info!("Block {} includes: {} from network {}", 
                          block.height, tx.transaction_data, tx.tenant_network);
                }
                
                let success = bc.add_block(block.clone());
                if success {
                    Some(block)
                } else {
                    warn!("Failed to add block {} to chain", block.height);
                    None
                }
            } else {
                info!("No transactions to mine");
                None
            }
        };
        
        if let Some(block) = new_block {
            info!("✅ Enterprise block {} created immediately for tenant transactions", block.height);
            
            // Broadcast to other validators (future consensus)
            self.broadcast_block_to_peers(&block).await;
            
            true
        } else {
            false
        }
    }
    
    // Simple heartbeat loop (no periodic block creation)
    async fn heartbeat_loop(
        blockchain: Arc<RwLock<EnterpriseBlockchain>>, 
        validator_id: String
    ) {
        let mut timer = interval(Duration::from_secs(30));
        
        loop {
            timer.tick().await;
            
            // Just update heartbeat - no block creation
            {
                let mut bc = blockchain.write().await;
                bc.update_validator_heartbeat(validator_id.clone());
                bc.cleanup_stale_validators();
            }
            
            info!("Validator {} heartbeat - ready for transactions", validator_id);
        }
    }
    
    // Future: Broadcast block to other validators for consensus
    async fn broadcast_block_to_peers(&self, block: &crate::enterprise_bc::EnterpriseBlock) {
        info!("Broadcasting block {} to {} peers", block.height, self.peers.len());
        
        // Future implementation:
        // - Send block to other validators
        // - Wait for consensus confirmation
        // - Handle conflicts and forks
        
        for peer in &self.peers {
            info!("Would broadcast to peer: {}", peer);
        }
    }
    
    pub async fn get_blockchain_status(&self) -> serde_json::Value {
        let blockchain = self.blockchain.read().await;
        blockchain.get_blockchain_info()
    }
}

// NEW: Updated API handler for immediate block creation
pub async fn handle_tenant_blockchain_update(
    update: TenantBlockchainUpdate,
    validator: Arc<Validator>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received blockchain update from network: {}", update.network_id);
    
    // Process immediately - create block right away
    let success = validator.process_tenant_update(update.clone()).await;
    
    if success {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": "Tenant transactions processed and block created immediately",
            "network_id": update.network_id,
            "new_blocks": update.new_blocks.len(),
            "immediate_processing": true
        })))
    } else {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Failed to process tenant transactions",
            "network_id": update.network_id
        })))
    }
}

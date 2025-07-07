use crate::blockchain::{Blockchain, TenantBlockchainUpdate};
use crate::enterprise_bc::api;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct Validator {
    pub id: String,
    pub port: u16,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub stake: u64,  // Keep u64 for stake amounts
    pub tracker_url: Option<String>,
}

impl Validator {
    pub async fn new(id: String, port: u16, initial_stake: u64) -> Self {
        let mut blockchain = Blockchain::new();
        
        // Add self as validator with initial stake - convert u64 to u32 for the interface
        blockchain.add_validator(id.clone(), initial_stake as u32);
        
        // Get tracker URL from environment for cross-network trading
        let tracker_url = std::env::var("TRACKER_URL").ok();
        if let Some(ref url) = tracker_url {
            info!("Tracker URL configured: {} (for cross-network trading)", url);
        } else {
            warn!("No TRACKER_URL provided - order book updates won't be broadcast");
        }
        
        Validator {
            id,
            port,
            blockchain: Arc::new(RwLock::new(blockchain)),
            stake: initial_stake,
            tracker_url,
        }
    }
    
    pub async fn start(self) {
        info!("Starting validator {} on port {} with stake {}", self.id, self.port, self.stake);
        
        let blockchain = self.blockchain.clone();
        let validator_id = self.id.clone();
        
        // Start PoS validation loop
        let validation_blockchain = blockchain.clone();
        let validation_validator_id = validator_id.clone();
        let validation_handle = tokio::spawn(async move {
            Self::pos_validation_loop(validation_blockchain, validation_validator_id).await;
        });
        
        // Start API server
        let api_blockchain = blockchain.clone();
        let api_handle = tokio::spawn(async move {
            api::start_api_server(self.port, api_blockchain).await;
        });
        
        info!("Validator {} ready for Proof of Stake consensus", self.id);
        
        // Wait for all services
        tokio::select! {
            _ = validation_handle => warn!("Validation stopped"),
            _ = api_handle => warn!("API server stopped"),
        }
    }
    
    // Proof of Stake validation loop
    async fn pos_validation_loop(blockchain: Arc<RwLock<Blockchain>>, validator_id: String) {
        let mut timer = interval(Duration::from_secs(10));
        
        loop {
            timer.tick().await;
            
            // Try to create a block if we have pending transactions
            {
                let mut bc = blockchain.write().await;
                let pending_count = bc.get_pending_count();  // Now returns u32
                if pending_count > 0 {
                    if bc.mine_block() {
                        info!("Validator {} created block via Proof of Stake", validator_id);
                    }
                }
            }
        }
    }
    
    // Process tenant blockchain updates (keep for dashboard compatibility)
    pub async fn process_tenant_update(&self, update: TenantBlockchainUpdate) -> bool {
        info!("Processing tenant update from network: {}", update.network_id);
        
        {
            let mut bc = self.blockchain.write().await;
            
            // Add tenant blocks for dashboard display
            bc.add_tenant_blocks(&update);
            
            info!("Stored {} blocks from {} for enterprise tracking", 
                  update.new_blocks.len(), update.network_id);
        }
        
        // Broadcast update back to tracker for cross-network trading
        if let Some(ref tracker_url) = self.tracker_url {
            self.broadcast_to_tracker(&update, tracker_url).await;
        }
        
        true
    }
    
    // Broadcast blockchain updates back to tracker for cross-network trading
    async fn broadcast_to_tracker(&self, update: &TenantBlockchainUpdate, tracker_url: &str) {
        let client = reqwest::Client::new();
        let url = format!("{}/api/enterprise-update", tracker_url);
        
        let enterprise_update = serde_json::json!({
            "type": "enterprise_blockchain_update",
            "network_id": update.network_id,
            "blocks": update.new_blocks,
            "timestamp": update.timestamp,
            "validator_id": self.id
        });
        
        match client.post(&url).json(&enterprise_update).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Broadcast update back to tracker for network: {}", update.network_id);
                } else {
                    warn!("Failed to broadcast to tracker: HTTP {}", response.status());
                }
            }
            Err(e) => {
                warn!("Failed to broadcast to tracker: {}", e);
            }
        }
    }
    
    pub async fn get_blockchain_status(&self) -> serde_json::Value {
        let blockchain = self.blockchain.read().await;
        
        // Get tenant summaries for dashboard
        let tenant_summaries = blockchain.get_tenant_summaries();
        let total_tenant_blocks = blockchain.get_recent_tenant_blocks(1000).len();
        
        // Use the new u32 methods
        let pending_count = blockchain.get_pending_count();  // u32
        let validator_count = blockchain.get_validator_count();  // u32
        
        serde_json::json!({
            "validator_id": self.id,
            "stake": self.stake,
            "height": total_tenant_blocks,
            "total_blocks": total_tenant_blocks,
            "total_transactions": pending_count,
            "active_validators": validator_count,
            "active_tenants": tenant_summaries.len(),
            "tenant_blocks": total_tenant_blocks,
            "consensus": "proof_of_stake",
            "chain_health": "healthy",
            "validator_status": "online"
        })
    }
}

// Keep the tenant blockchain update handler for dashboard
pub async fn handle_tenant_blockchain_update(
    update: TenantBlockchainUpdate,
    validator: Arc<Validator>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received blockchain update from network: {}", update.network_id);
    
    // Process the update for enterprise tracking
    let success = validator.process_tenant_update(update.clone()).await;
    
    if success {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": "Tenant blockchain update processed",
            "network_id": update.network_id,
            "new_blocks": update.new_blocks.len(),
            "immediate_processing": true
        })))
    } else {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Failed to process tenant blockchain update",
            "network_id": update.network_id
        })))
    }
}

use crate::enterprise_bc::{EnterpriseBlockchain, ConsensusEngine};
use crate::enterprise_bc::api;
use crate::common::types::TenantUpdate;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct Validator {
    pub id: String,
    pub port: u16,
    pub blockchain: Arc<RwLock<EnterpriseBlockchain>>,
    pub consensus: ConsensusEngine,
    pub peers: Vec<String>,
}

impl Validator {
    pub async fn new(id: String, port: u16, peers: Vec<String>) -> Self {
        let blockchain = Arc::new(RwLock::new(EnterpriseBlockchain::new(id.clone())));
        let consensus = ConsensusEngine::new(id.clone(), peers.clone());
        
        Validator {
            id,
            port,
            blockchain,
            consensus,
            peers,
        }
    }
    
    pub async fn start(self) {
        info!("Starting validator {} on port {}", self.id, self.port);
        
        let blockchain = self.blockchain.clone();
        let consensus = self.consensus.clone();
        let validator_id = self.id.clone();
        
        // Start consensus engine
        let consensus_blockchain = blockchain.clone();
        let consensus_handle = tokio::spawn(async move {
            consensus.start_consensus_loop(consensus_blockchain).await;
        });
        
        // Start block production timer
        let production_blockchain = blockchain.clone();
        let production_validator_id = validator_id.clone();
        let production_handle = tokio::spawn(async move {
            Self::block_production_loop(production_blockchain, production_validator_id).await;
        });
        
        // Start API server
        let api_blockchain = blockchain.clone();
        let api_handle = tokio::spawn(async move {
            api::start_api_server(self.port, api_blockchain).await;
        });
        
        info!("Validator {} fully started", self.id);
        
        // Wait for all services
        tokio::select! {
            _ = consensus_handle => warn!("Consensus engine stopped"),
            _ = production_handle => warn!("Block production stopped"),
            _ = api_handle => warn!("API server stopped"),
        }
    }
    
    async fn block_production_loop(
        blockchain: Arc<RwLock<EnterpriseBlockchain>>, 
        validator_id: String
    ) {
        let mut timer = interval(Duration::from_secs(30)); // New block every 30 seconds
        
        loop {
            timer.tick().await;
             // ADD HEARTBEAT UPDATE
            {
                 let mut bc = blockchain.write().await;
                 bc.update_validator_heartbeat(validator_id.clone());
                 bc.cleanup_stale_validators(); // Remove stale validators
            }
            
            let should_produce = {
                let bc = blockchain.read().await;
                !bc.pending_tenant_updates.is_empty()
            };
            
            if should_produce {
                info!("Validator {} attempting to produce new block", validator_id);
                
                let new_block = {
                    let mut bc = blockchain.write().await;
                    bc.create_new_block()
                };
                
                // In a real implementation, this would go through consensus
                // For simplicity, we'll just add it directly
                {
                    let mut bc = blockchain.write().await;
                    if bc.add_block(new_block.clone()) {
                        info!("Block {} added by validator {}", new_block.height, validator_id);
                        info!("Block contains {} tenant summaries", new_block.tenant_summaries.len());
                    }
                }
            }
        }
    }
    
    pub async fn add_tenant_update(&self, update: TenantUpdate) {
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_tenant_update(update);
        info!("Added tenant update for: {}", blockchain.pending_tenant_updates.last().unwrap().tenant_id);
    }
    
    pub async fn get_blockchain_status(&self) -> serde_json::Value {
        let blockchain = self.blockchain.read().await;
        blockchain.get_blockchain_info()
    }
}

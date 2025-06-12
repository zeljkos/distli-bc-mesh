use crate::enterprise_bc::{EnterpriseBlockchain, EnterpriseBlock};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

#[derive(Clone)]
pub struct ConsensusEngine {
    pub validator_id: String,
    pub peers: Vec<String>,
    pub current_leader: usize,
}

impl ConsensusEngine {
    pub fn new(validator_id: String, peers: Vec<String>) -> Self {
        ConsensusEngine {
            validator_id,
            peers,
            current_leader: 0,
        }
    }
    
    pub async fn start_consensus_loop(
        self, 
        _blockchain: Arc<RwLock<EnterpriseBlockchain>>
    ) {
        info!("Starting consensus engine for validator: {}", self.validator_id);
        
        let mut timer = interval(Duration::from_secs(10));
        
        loop {
            timer.tick().await;
            
            let current_leader = self.get_current_leader();
            
            if current_leader == self.validator_id {
                info!("Validator {} is current leader", self.validator_id);
                self.rotate_leader().await;
            } else {
                info!("Validator {} is follower, leader is: {}", self.validator_id, current_leader);
            }
        }
    }
    
    fn get_current_leader(&self) -> String {
        if self.peers.is_empty() {
            return self.validator_id.clone();
        }
        
        let leader_peer = &self.peers[self.current_leader % self.peers.len()];
        leader_peer.split(':').next().unwrap_or(leader_peer).to_string()
    }
    
    async fn rotate_leader(&self) {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
    
    pub async fn validate_block(&self, block: &EnterpriseBlock) -> bool {
        if block.height == 0 {
            return true;
        }
        
        let expected_leader = self.get_current_leader();
        if block.validator != expected_leader {
            warn!("Block validator {} does not match expected leader {}", 
                  block.validator, expected_leader);
            return false;
        }
        
        if block.hash.is_empty() || block.previous_hash.is_empty() {
            warn!("Invalid block structure");
            return false;
        }
        
        info!("Block {} validated successfully", block.height);
        true
    }
    
    pub async fn broadcast_block(&self, _block: &EnterpriseBlock) {
        info!("Broadcasting block to {} peers", self.peers.len());
    }
}

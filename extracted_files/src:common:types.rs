// Shared types used across tracker and enterprise blockchain
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "join_network")]
    JoinNetwork { network_id: String },
    
    #[serde(rename = "peers")]
    Peers { peers: Vec<String> },
    
    #[serde(rename = "offer")]
    Offer { 
        target: String, 
        offer: serde_json::Value 
    },
    
    #[serde(rename = "answer")]
    Answer { 
        target: String, 
        answer: serde_json::Value 
    },
    
    #[serde(rename = "candidate")]
    Candidate { 
        target: String, 
        candidate: serde_json::Value 
    },
    
    #[serde(rename = "block")]
    Block { block: crate::common::blockchain::Block },
    
    #[serde(rename = "transaction")]
    Transaction { transaction: crate::common::blockchain::Transaction },
    
    #[serde(rename = "network_info")]
    NetworkInfo { 
        network_id: String, 
        peer_count: usize 
    },
    
    #[serde(rename = "network_list_update")]
    NetworkListUpdate {
        networks: Vec<serde_json::Value>
    },
}

#[derive(Debug, Clone)]
pub struct NetworkPeer {
    pub peer_id: String,
    pub network_id: String,
    pub sender: tokio::sync::mpsc::UnboundedSender<Result<warp::ws::Message, warp::Error>>,
    pub joined_at: std::time::Instant,
}

// Shared between tracker and enterprise blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUpdate {
    pub tenant_id: String,
    pub blocks_added: u64,
    pub transactions_added: u64,
    pub new_messages: Vec<String>,
    pub active_peers: usize,
    pub timestamp: u64,
}

// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub network_id: String,
    pub peer_count: usize,
    pub block_count: u64,
    pub transaction_count: u64,
    pub last_activity: u64,
}

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
    
    #[serde(rename = "blockchain_update")]
    BlockchainUpdate {
        network_id: String,
        peer_id: String,
        new_blocks: Vec<serde_json::Value>,
        timestamp: u64,
    },
    #[serde(rename = "cross_network_trade")]
    CrossNetworkTrade {
        trade_id: String,
        buyer_network: String,
        seller_network: String,
        asset: String,
        quantity: f64,
        price: f64,
        buyer_order_id: u64,
        seller_order_id: u64,
        timestamp: u64,
    },

    #[serde(rename = "order_book_sync")]
    OrderBookSync {
        orders: Vec<serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossNetworkTradeNotification {
    pub trade_id: String,
    pub buyer_network: String,
    pub seller_network: String,
    pub asset: String,
    pub quantity: f64,
    pub price: f64,
    pub buyer_order_id: u64,
    pub seller_order_id: u64,
    pub timestamp: u64,
}

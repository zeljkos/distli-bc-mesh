use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub data: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
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
    Block { block: Block },
    
    #[serde(rename = "transaction")]
    Transaction { transaction: Transaction },
}

impl Block {
    pub fn new(id: u64, data: String, prev_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut block = Block {
            id,
            hash: String::new(),
            prev_hash,
            timestamp,
            data,
            nonce: 0,
        };
        
        block.mine();
        block
    }
    
    pub fn genesis() -> Self {
        Block::new(0, "Genesis".to_string(), "0".to_string())
    }
    
    fn calculate_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let input = format!("{}{}{}{}{}", 
            self.id, self.prev_hash, self.timestamp, self.data, self.nonce);
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(hash)
    }
    
    fn mine(&mut self) {
        while !self.hash.starts_with("00") {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

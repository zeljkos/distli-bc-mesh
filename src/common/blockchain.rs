// Shared blockchain components used by both tenant and enterprise blockchains
use serde::{Deserialize, Serialize};
use crate::common::{crypto::hash_data, time::current_timestamp};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;


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

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: VecDeque<Transaction>,
    pub storage_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainData {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
}

impl Block {
    pub fn new(id: u64, data: String, prev_hash: String) -> Self {
        let timestamp = current_timestamp();
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
    
    pub fn calculate_hash(&self) -> String {
        let input = format!("{}{}{}{}{}", 
            self.id, self.prev_hash, self.timestamp, self.data, self.nonce);
        hash_data(&input)
    }
    
    pub fn mine(&mut self) {
        while !self.hash.starts_with("00") {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending: VecDeque::new(),
            storage_path: None,
        };
        blockchain.chain.push(Block::genesis());
        blockchain
    }
    
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending.push_back(transaction);
        self.save_to_disk();
    }
    
    pub fn mine_block(&mut self, data: String) -> Block {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.id + 1,
            data,
            last_block.hash.clone()
        );
        
        self.chain.push(new_block.clone());
        self.save_to_disk();
        new_block
    }
    
    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.chain.last().unwrap();
        
        if block.id == last_block.id + 1 && block.prev_hash == last_block.hash {
            self.chain.push(block);
            self.save_to_disk();
            true
        } else {
            false
        }
    }
    
    pub fn get_latest(&self) -> &Block {
        self.chain.last().unwrap()
    }
    
    pub fn height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }
    pub fn new_with_storage(storage_path: String) -> Self {
        let mut blockchain = Self::new();
        blockchain.storage_path = Some(storage_path);
        blockchain.load_from_disk();
        blockchain
    }

    pub fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            let data = BlockchainData {
                chain: self.chain.clone(),
                pending: self.pending.iter().cloned().collect(),
            };

           if let Ok(json) = serde_json::to_string_pretty(&data) {
                if let Some(parent) = Path::new(path).parent() {
                    let _ = fs::create_dir_all(parent);
               }
               let _ = fs::write(path, json);
            }
        }
    }

    pub fn load_from_disk(&mut self) {
        if let Some(path) = &self.storage_path {
            if Path::new(path).exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(data) = serde_json::from_str::<BlockchainData>(&content) {
                        if !data.chain.is_empty() {
                            self.chain = data.chain;
                            self.pending = data.pending.into_iter().collect();
                        }
                    }
                }
            }
        }
    }
}

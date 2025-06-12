// Shared blockchain components used by both tenant and enterprise blockchains
use serde::{Deserialize, Serialize};
use crate::common::{crypto::hash_data, time::current_timestamp};
use std::collections::VecDeque;

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
        };
        blockchain.chain.push(Block::genesis());
        blockchain
    }
    
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending.push_back(transaction);
    }
    
    pub fn mine_block(&mut self, data: String) -> Block {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.id + 1,
            data,
            last_block.hash.clone()
        );
        
        self.chain.push(new_block.clone());
        new_block
    }
    
    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.chain.last().unwrap();
        
        if block.id == last_block.id + 1 && block.prev_hash == last_block.hash {
            self.chain.push(block);
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
}

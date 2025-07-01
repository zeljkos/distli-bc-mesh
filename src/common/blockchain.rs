// src/common/blockchain.rs - Enhanced with real trading logic
use serde::{Deserialize, Serialize};
use crate::common::{crypto::hash_data, time::current_timestamp};
use crate::common::contracts::{ContractVM, SmartContract, ContractCall, ContractResult, create_trading_contract};
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
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "message")]
    Message { 
        id: String,
        data: String,
        timestamp: u64,
        sender: String,
    },
    #[serde(rename = "contract_deploy")]
    ContractDeploy {
        id: String,
        contract: SmartContract,
        timestamp: u64,
        sender: String,
    },
    #[serde(rename = "contract_call")]
    ContractCall {
        id: String,
        call: ContractCall,
        result: Option<ContractResult>,
        timestamp: u64,
        sender: String,
    },
}

impl Transaction {
    pub fn new_message(data: String, sender: String) -> Self {
        Transaction::Message {
            id: format!("msg_{}", current_timestamp()),
            data,
            timestamp: current_timestamp(),
            sender,
        }
    }

    pub fn new_contract_deploy(contract: SmartContract, sender: String) -> Self {
        Transaction::ContractDeploy {
            id: format!("deploy_{}", current_timestamp()),
            contract,
            timestamp: current_timestamp(),
            sender,
        }
    }

    pub fn new_contract_call(call: ContractCall, sender: String) -> Self {
        Transaction::ContractCall {
            id: format!("call_{}", current_timestamp()),
            call,
            result: None,
            timestamp: current_timestamp(),
            sender,
        }
    }

    pub fn get_summary(&self) -> String {
        match self {
            Transaction::Message { data, sender, .. } => format!("💬 {}: {}", sender, data),
            Transaction::ContractDeploy { contract, sender, .. } => format!("📄 {}: Deployed contract '{}'", sender, contract.name),
            Transaction::ContractCall { result, sender, .. } => {
                if let Some(res) = result {
                    if res.success {
                        if let Some(msg) = res.result.get("message") {
                            format!("🔧 {}: {}", sender, msg.as_str().unwrap_or("Contract executed"))
                        } else {
                            format!("🔧 {}: Contract call succeeded", sender)
                        }
                    } else {
                        format!("❌ {}: Contract call failed", sender)
                    }
                } else {
                    format!("🔧 {}: Contract call pending", sender)
                }
            }
        }
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: VecDeque<Transaction>,
    pub storage_path: Option<String>,
    pub contract_vm: ContractVM,
    pub offline_orders: Vec<OfflineOrder>, // New: Store offline orders
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineOrder {
    pub id: String,
    pub order_type: String, // "buy" or "sell"
    pub asset: String,
    pub quantity: f64,
    pub price: f64,
    pub trader: String,
    pub timestamp: u64,
    pub created_offline: bool,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending: VecDeque::new(),
            storage_path: None,
            contract_vm: ContractVM::new(),
            offline_orders: Vec::new(),
        };
        
        let genesis = Block::genesis();
        blockchain.chain.push(genesis);
        
        // Deploy trading contract automatically
        let trading_contract = create_trading_contract("system".to_string());
        let _ = blockchain.contract_vm.deploy_contract(trading_contract.clone());
        
        blockchain
    }
    
    pub fn new_with_storage(storage_path: String) -> Self {
        let mut blockchain = Self::new();
        blockchain.storage_path = Some(storage_path);
        blockchain.load_from_disk();
        blockchain
    }

    // Add different types of transactions
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending.push_back(transaction);
        self.save_to_disk();
    }

    pub fn add_message(&mut self, data: String, sender: String) -> Transaction {
        let transaction = Transaction::new_message(data, sender);
        self.pending.push_back(transaction.clone());
        self.save_to_disk();
        transaction
    }

    pub fn deploy_contract(&mut self, contract: SmartContract, sender: String) -> Result<Transaction, String> {
        // Deploy to VM first
        self.contract_vm.deploy_contract(contract.clone())
            .map_err(|e| format!("Failed to deploy contract: {}", e))?;
        
        let transaction = Transaction::new_contract_deploy(contract, sender);
        self.pending.push_back(transaction.clone());
        self.save_to_disk();
        Ok(transaction)
    }

    pub fn call_contract(&mut self, mut call: ContractCall, sender: String) -> Transaction {
        call.caller = sender.clone();
        
        // Execute the contract call immediately
        let result = self.contract_vm.call_contract(call.clone());
        
        let mut transaction = Transaction::new_contract_call(call, sender);
        
        // Update transaction with result
        if let Transaction::ContractCall { result: ref mut tx_result, .. } = &mut transaction {
            *tx_result = Some(result);
        }
        
        self.pending.push_back(transaction.clone());
        self.save_to_disk();
        transaction
    }

    // New: Add offline order functionality
    pub fn add_offline_order(&mut self, order_type: String, asset: String, quantity: f64, price: f64, trader: String) -> OfflineOrder {
        let order = OfflineOrder {
            id: format!("offline_{}_{}", order_type, current_timestamp()),
            order_type,
            asset,
            quantity,
            price,
            trader,
            timestamp: current_timestamp(),
            created_offline: true,
        };
        
        self.offline_orders.push(order.clone());
        self.save_to_disk();
        order
    }

    // New: Execute offline orders when reconnected
    pub fn execute_offline_orders(&mut self) -> Vec<Transaction> {
        let mut executed_transactions = Vec::new();
        
        // Collect offline orders first to avoid borrow checker issues
        let orders_to_execute: Vec<OfflineOrder> = self.offline_orders.drain(..).collect();
        
        for order in orders_to_execute {
            let call = ContractCall {
                contract_id: "trading_contract".to_string(),
                function: order.order_type.clone(),
                params: serde_json::json!({
                    "asset": order.asset,
                    "quantity": order.quantity,
                    "price": order.price
                }),
                caller: order.trader.clone(),
                gas_limit: 100,
            };
            
            let tx = self.call_contract(call, order.trader);
            executed_transactions.push(tx);
        }
        
        if !executed_transactions.is_empty() {
            self.save_to_disk();
        }
        
        executed_transactions
    }

    // New: Get pending offline orders
    pub fn get_offline_orders(&self) -> &Vec<OfflineOrder> {
        &self.offline_orders
    }

    // New: Cancel offline order
    pub fn cancel_offline_order(&mut self, order_id: &str, trader: &str) -> bool {
        if let Some(pos) = self.offline_orders.iter().position(|o| o.id == order_id && o.trader == trader) {
            self.offline_orders.remove(pos);
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    // Existing blockchain methods with transaction support
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

    pub fn mine_pending_block(&mut self) -> Option<Block> {
        if self.pending.is_empty() {
            return None;
        }

        let last_block = self.chain.last().unwrap();
        let transactions: Vec<Transaction> = self.pending.drain(..).collect();
        
        // Create summary data for backward compatibility
        let data = transactions.iter()
            .map(|tx| tx.get_summary())
            .collect::<Vec<_>>()
            .join(", ");
        
        let mut block = Block {
            id: last_block.id + 1,
            hash: String::new(),
            prev_hash: last_block.hash.clone(),
            timestamp: current_timestamp(),
            data,
            nonce: 0,
            transactions,
        };
        
        block.mine();
        self.chain.push(block.clone());
        self.save_to_disk();
        Some(block)
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.chain.last().unwrap();
        
        if block.id == last_block.id + 1 && block.prev_hash == last_block.hash {
            // Process any contract transactions in the block
            for transaction in &block.transactions {
                if let Transaction::ContractDeploy { contract, .. } = transaction {
                    let _ = self.contract_vm.deploy_contract(contract.clone());
                } else if let Transaction::ContractCall { call, .. } = transaction {
                    // Re-execute contract calls when receiving blocks from peers
                    self.contract_vm.call_contract(call.clone());
                }
            }
            
            self.chain.push(block);
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    // Contract query methods
    pub fn get_contract_state(&self, contract_id: &str) -> Option<serde_json::Value> {
        self.contract_vm.get_contract(contract_id)
            .map(|contract| contract.state.clone())
    }

    pub fn list_contracts(&self) -> Vec<&SmartContract> {
        self.contract_vm.list_contracts()
    }

    // Get trading data for UI
    pub fn get_order_book(&mut self, asset: Option<&str>) -> serde_json::Value {
        let call = ContractCall {
            contract_id: "trading_contract".to_string(),
            function: "getOrderBook".to_string(),
            params: if let Some(asset) = asset {
                serde_json::json!({ "asset": asset })
            } else {
                serde_json::json!({})
            },
            caller: "system".to_string(),
            gas_limit: 100,
        };
        
        let result = self.contract_vm.call_contract(call);
        result.result
    }

    pub fn get_recent_trades(&mut self, asset: Option<&str>, limit: Option<u64>) -> serde_json::Value {
        let mut params = serde_json::json!({});
        if let Some(asset) = asset {
            params["asset"] = serde_json::Value::String(asset.to_string());
        }
        if let Some(limit) = limit {
            params["limit"] = serde_json::Value::Number(serde_json::Number::from(limit));
        }

        let call = ContractCall {
            contract_id: "trading_contract".to_string(),
            function: "getTrades".to_string(),
            params,
            caller: "system".to_string(),
            gas_limit: 100,
        };
        
        let result = self.contract_vm.call_contract(call);
        result.result
    }

    // Get trading summary for display
    pub fn get_trading_summary(&mut self) -> serde_json::Value {
        let order_book = self.get_order_book(None);
        let recent_trades = self.get_recent_trades(None, Some(10));
        
        let bids_count = order_book.get("bids")
            .and_then(|b| b.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        let asks_count = order_book.get("asks")
            .and_then(|a| a.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        let trades_count = recent_trades.get("count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        serde_json::json!({
            "order_book": order_book,
            "recent_trades": recent_trades,
            "offline_orders": self.offline_orders,
            "summary": {
                "total_bids": bids_count,
                "total_asks": asks_count,
                "total_trades": trades_count,
                "offline_orders_count": self.offline_orders.len()
            }
        })
    }

    // Existing methods remain the same
    pub fn get_latest(&self) -> &Block {
        self.chain.last().unwrap()
    }
    
    pub fn height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }

    pub fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            let data = BlockchainData {
                chain: self.chain.clone(),
                pending: self.pending.iter().cloned().collect(),
                contracts: self.contract_vm.list_contracts().iter().map(|c| (*c).clone()).collect(),
                offline_orders: Some(self.offline_orders.clone()),
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
                            self.offline_orders = data.offline_orders.unwrap_or_default();
                            
                            // Restore contracts
                            for contract in data.contracts {
                                let _ = self.contract_vm.deploy_contract(contract);
                            }
                        }
                    }
                }
            }
        }
    }
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
            transactions: vec![],
        };
        block.mine();
        block
    }
    
    pub fn genesis() -> Self {
        Block {
            id: 0,
            hash: "000genesis".to_string(),
            prev_hash: "0".to_string(),
            timestamp: current_timestamp(),
            data: "Genesis Block".to_string(),
            nonce: 0,
            transactions: vec![],
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainData {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub contracts: Vec<SmartContract>,
    pub offline_orders: Option<Vec<OfflineOrder>>, // Optional for backward compatibility
}

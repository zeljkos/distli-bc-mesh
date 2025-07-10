use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Shared blockchain types - work for both native and WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u32,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub transactions: Vec<Transaction>,
    pub stake_weight: u64,
    // Optional fields for WASM/Native compatibility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub tx_type: TransactionType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Stake,
    Trading { asset: String, quantity: u64, price: u64 },
    Message { content: String },
    ContractDeploy { contract_name: String },
    ContractCall { function: String, params: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
    pub active: bool,
}

// Enterprise types (native only)
#[cfg(feature = "native")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockchainUpdate {
    pub network_id: String,
    pub peer_id: String,
    pub new_blocks: Vec<TenantBlockData>,
    pub timestamp: u64,
}

#[cfg(feature = "native")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockData {
    pub block_id: u32,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub previous_hash: String,
    pub network_id: String,  // Add this field
}

// Smart contract support (native only)
#[cfg(feature = "native")]
use std::fs;
#[cfg(feature = "native")]
use std::path::Path;

#[cfg(feature = "native")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: String,
    pub name: String,
    pub code: String,
    pub state: serde_json::Value,
    pub owner: String,
    pub created_at: u64,
}

#[cfg(feature = "native")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_id: String,
    pub function: String,
    pub params: serde_json::Value,
    pub caller: String,
    pub gas_limit: u32,
}

#[cfg(feature = "native")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub gas_used: u32,
    pub state_changes: Option<serde_json::Value>,
    pub error: Option<String>,
}

// UNIFIED BLOCKCHAIN - works for both native and WASM
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    validators: HashMap<String, Validator>,
    total_stake: u64,
    
    // Native-only features
    #[cfg(feature = "native")]
    storage_path: Option<String>,
    #[cfg(feature = "native")]
    contracts: HashMap<String, SmartContract>,
    #[cfg(feature = "native")]
    tenant_blocks: Vec<TenantBlockData>,
}

// WASM-compatible methods only
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Blockchain {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        let mut blockchain = Self {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            validators: HashMap::new(),
            total_stake: 0,
            #[cfg(feature = "native")]
            storage_path: None,
            #[cfg(feature = "native")]
            contracts: HashMap::new(),
            #[cfg(feature = "native")]
            tenant_blocks: Vec::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }


        // Modified to create immediate block for messages
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_message_and_mine(&mut self, message: String, sender: String) -> String {
        // Check for duplicate content from same sender
        for existing_tx in &self.pending_transactions {
            if let TransactionType::Message { content } = &existing_tx.tx_type {
                if content == &message && existing_tx.from == sender {
                    return existing_tx.id.clone(); // Return existing ID
                }
            }
        }

        let tx_id = format!("msg_{}", Self::current_timestamp());
        let tx = Transaction {
            id: tx_id.clone(),
            from: sender.clone(),
            to: "broadcast".to_string(),
            amount: 0,
            tx_type: TransactionType::Message { content: message },
            timestamp: Self::current_timestamp(),
        };

        // Add transaction and immediately mine block
        self.pending_transactions.push(tx);

        // Only mine if we have validators and this sender can mine
        if !self.validators.is_empty() {
            self.mine_block();
        }

        #[cfg(feature = "native")]
        self.save_to_disk();

        tx_id
    }
        // Modified mine_block to return the created block for broadcasting
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn mine_block_and_get(&mut self) -> String {
        if self.pending_transactions.is_empty() {
            return "{}".to_string();
        }

        let validator = self.select_validator();
        if validator.is_none() {
            return "{}".to_string();
        }

        let last_block = self.chain.last().unwrap();
        let validator_addr = validator.unwrap();
        let stake_weight = self.validators.get(&validator_addr).unwrap().stake;

        let mut block = Block {
            height: last_block.height + 1,
            hash: String::new(),
            previous_hash: last_block.hash.clone(),
            timestamp: Self::current_timestamp(),
            validator: validator_addr,
            transactions: self.pending_transactions.clone(),
            stake_weight,
            nonce: None,
            data: None,
        };

        // Calculate hash
        block.hash = self.calculate_hash(&block);

        // For native builds, do proof-of-work mining
        #[cfg(feature = "native")]
        self.mine_block_native(&mut block);

        self.chain.push(block.clone());
        self.pending_transactions.clear();

        #[cfg(feature = "native")]
        self.save_to_disk();

        // Return the block as JSON for broadcasting
        serde_json::to_string(&block).unwrap_or_default()
    }


    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_validator(&mut self, address: String, stake: u32) {
        let stake_u64 = stake as u64;  // Convert u32 to u64
        let validator = Validator {
            address: address.clone(),
            stake: stake_u64,
            active: true,
        };
        self.total_stake += stake_u64;
        self.validators.insert(address, validator);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_transaction(&mut self, from: String, to: String, amount: u32) -> String {
        let amount_u64 = amount as u64;  // Convert u32 to u64
        let tx = Transaction {
            id: format!("tx_{}", Self::current_timestamp()),
            from,
            to,
            amount: amount_u64,
            tx_type: TransactionType::Transfer,
            timestamp: Self::current_timestamp(),
        };
        let tx_id = tx.id.clone();
        self.pending_transactions.push(tx);
        #[cfg(feature = "native")]
        self.save_to_disk();
        tx_id
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_message(&mut self, message: String, sender: String) -> String {
        // Check for duplicate content from same sender
        for existing_tx in &self.pending_transactions {
            if let TransactionType::Message { content } = &existing_tx.tx_type {
                if content == &message && existing_tx.from == sender {
                    return existing_tx.id.clone(); // Return existing ID
                }
            }
        }
        
        let tx_id = format!("msg_{}", Self::current_timestamp());
        let tx = Transaction {
            id: tx_id.clone(),
            from: sender.clone(),
            to: "broadcast".to_string(),
            amount: 0,
            tx_type: TransactionType::Message { content: message },
            timestamp: Self::current_timestamp(),
        };
        self.pending_transactions.push(tx);
        #[cfg(feature = "native")]
        self.save_to_disk();
        tx_id
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn deploy_trading_contract(&mut self, owner: String) -> bool {
        let tx_id = format!("contract_{}", Self::current_timestamp());
        let tx = Transaction {
            id: tx_id,
            from: owner,
            to: "trading_contract".to_string(),
            amount: 0,
            tx_type: TransactionType::ContractDeploy { 
                contract_name: "trading_contract".to_string() 
            },
            timestamp: Self::current_timestamp(),
        };
        self.pending_transactions.push(tx);
        #[cfg(feature = "native")]
        self.save_to_disk();
        true
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn call_contract_buy(&mut self, asset: String, quantity: f64, price: f64, sender: String) -> String {
        let tx_id = format!("buy_{}", Self::current_timestamp());
        let quantity_u64 = quantity as u64;  // Convert f64 to u64
        let price_u64 = price as u64;        // Convert f64 to u64
        let tx = Transaction {
            id: tx_id.clone(),
            from: sender,
            to: "trading_contract".to_string(),
            amount: (price * quantity) as u64,
            tx_type: TransactionType::Trading { 
                asset: asset.clone(), 
                quantity: quantity_u64, 
                price: price_u64 
            },
            timestamp: Self::current_timestamp(),
        };
        self.pending_transactions.push(tx);
        #[cfg(feature = "native")]
        self.save_to_disk();
        
        format!("{{\"status\": \"success\", \"message\": \"Buy order placed: {} {} @ {}\", \"orderId\": \"{}\"}}", 
                quantity, asset, price, tx_id)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn call_contract_sell(&mut self, asset: String, quantity: f64, price: f64, sender: String) -> String {
        let tx_id = format!("sell_{}", Self::current_timestamp());
        let quantity_u64 = quantity as u64;  // Convert f64 to u64
        let price_u64 = price as u64;        // Convert f64 to u64
        let tx = Transaction {
            id: tx_id.clone(),
            from: sender,
            to: "trading_contract".to_string(),
            amount: (price * quantity) as u64,
            tx_type: TransactionType::Trading { 
                asset: asset.clone(), 
                quantity: quantity_u64, 
                price: price_u64 
            },
            timestamp: Self::current_timestamp(),
        };
        self.pending_transactions.push(tx);
        #[cfg(feature = "native")]
        self.save_to_disk();
        
        format!("{{\"status\": \"success\", \"message\": \"Sell order placed: {} {} @ {}\", \"orderId\": \"{}\"}}", 
                quantity, asset, price, tx_id)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_contract_order_book(&self) -> String {
        let mut buy_orders = Vec::new();
        let mut sell_orders = Vec::new();
        
        for tx in &self.pending_transactions {
            if let TransactionType::Trading { asset, quantity, price } = &tx.tx_type {
                let order = format!(
                    "{{\"asset\": \"{}\", \"quantity\": {}, \"price\": {}, \"trader\": \"{}\", \"timestamp\": {}}}",
                    asset, quantity, price, &tx.from[..8.min(tx.from.len())], tx.timestamp
                );
                
                if tx.to == "trading_contract" {
                    buy_orders.push(order);
                } else {
                    sell_orders.push(order);
                }
            }
        }
        
        for block in &self.chain {
            for tx in &block.transactions {
                if let TransactionType::Trading { asset, quantity, price } = &tx.tx_type {
                    let order = format!(
                        "{{\"asset\": \"{}\", \"quantity\": {}, \"price\": {}, \"trader\": \"{}\", \"timestamp\": {}}}",
                        asset, quantity, price, &tx.from[..8.min(tx.from.len())], tx.timestamp
                    );
                    
                    if tx.to == "trading_contract" {
                        buy_orders.push(order);
                    } else {
                        sell_orders.push(order);
                    }
                }
            }
        }
        
        format!("{{\"bids\": [{}], \"asks\": [{}]}}", 
                buy_orders.join(","), sell_orders.join(","))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_contract_trades(&self) -> String {
        let mut trades = Vec::new();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if let TransactionType::Trading { asset, quantity, price } = &tx.tx_type {
                    trades.push(format!(
                        "{{\"asset\": \"{}\", \"quantity\": {}, \"price\": {}, \"trader\": \"{}\", \"timestamp\": {}, \"type\": \"{}\"}}",
                        asset, quantity, price, &tx.from[..8.min(tx.from.len())], tx.timestamp,
                        if tx.to == "trading_contract" { "buy" } else { "sell" }
                    ));
                }
            }
        }
        
        for tx in &self.pending_transactions {
            if let TransactionType::Trading { asset, quantity, price } = &tx.tx_type {
                trades.push(format!(
                    "{{\"asset\": \"{}\", \"quantity\": {}, \"price\": {}, \"trader\": \"{}\", \"timestamp\": {}, \"type\": \"pending\"}}",
                    asset, quantity, price, &tx.from[..8.min(tx.from.len())], tx.timestamp
                ));
            }
        }
        
        format!("{{\"trades\": [{}]}}", trades.join(","))
    }

    // Proof of Stake mining
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn mine_block(&mut self) -> bool {
        if self.pending_transactions.is_empty() {
            return false;
        }

        let validator = self.select_validator();
        if validator.is_none() {
            return false;
        }

        let last_block = self.chain.last().unwrap();
        let validator_addr = validator.unwrap();
        let stake_weight = self.validators.get(&validator_addr).unwrap().stake;

        let mut block = Block {
            height: last_block.height + 1,
            hash: String::new(),
            previous_hash: last_block.hash.clone(),
            timestamp: Self::current_timestamp(),
            validator: validator_addr,
            transactions: self.pending_transactions.clone(),
            stake_weight,
            nonce: None,  // WASM-compatible: always None
            data: None,   // WASM-compatible: always None
        };

        // Calculate hash
        block.hash = self.calculate_hash(&block);
        
        // For native builds, do proof-of-work mining
        #[cfg(feature = "native")]
        self.mine_block_native(&mut block);

        self.chain.push(block);
        self.pending_transactions.clear();
        
        #[cfg(feature = "native")]
        self.save_to_disk();
        
        true
    }

    // Public getters - work for both WASM and native
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_chain_length(&self) -> u32 {
        self.chain.len() as u32
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_pending_count(&self) -> u32 {
        self.pending_transactions.len() as u32
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_validator_count(&self) -> u32 {
        self.validators.len() as u32
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_latest_block_json(&self) -> String {
        match self.chain.last() {
            Some(block) => serde_json::to_string(block).unwrap_or_default(),
            None => "{}".to_string(),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_transactions_json(&self) -> String {
        serde_json::to_string(&self.pending_transactions).unwrap_or_default()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_blockchain_summary(&self) -> String {
        let mut message_count = 0;
        let mut trading_count = 0;
        let mut transfer_count = 0;
        
        for tx in &self.pending_transactions {
            match &tx.tx_type {
                TransactionType::Message { .. } => message_count += 1,
                TransactionType::Trading { .. } => trading_count += 1,
                TransactionType::Transfer => transfer_count += 1,
                _ => {}
            }
        }
        
        for block in &self.chain {
            for tx in &block.transactions {
                match &tx.tx_type {
                    TransactionType::Message { .. } => message_count += 1,
                    TransactionType::Trading { .. } => trading_count += 1,
                    TransactionType::Transfer => transfer_count += 1,
                    _ => {}
                }
            }
        }
        
        format!(
            "{{\"blocks\": {}, \"pending\": {}, \"validators\": {}, \"messages\": {}, \"trades\": {}, \"transfers\": {}}}",
            self.chain.len(), self.pending_transactions.len(), self.validators.len(),
            message_count, trading_count, transfer_count
        )
    }

    // Additional methods for P2P sync support
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_p2p_transaction(&mut self, tx_json: String) -> bool {
        if let Ok(tx) = serde_json::from_str::<Transaction>(&tx_json) {
            self.pending_transactions.push(tx);
            #[cfg(feature = "native")]
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_p2p_block(&mut self, block_json: String) -> bool {
        if let Ok(block) = serde_json::from_str::<Block>(&block_json) {
            if self.validate_block(&block) {
                self.chain.push(block);
                #[cfg(feature = "native")]
                self.save_to_disk();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_sync_summary(&self) -> String {
        let current_height = self.chain.len();
        let pending_transactions = self.pending_transactions.len();
        let validators = self.validators.len();
        
        format!("{{\"current_height\": {}, \"last_sync_height\": {}, \"new_blocks\": 0, \"pending_transactions\": {}, \"validators\": {}}}", 
                current_height, current_height, pending_transactions, validators)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn prepare_enterprise_sync(&self) -> String {
        let new_blocks: Vec<&Block> = self.chain.iter().rev().take(10).collect();
        let pending_txs: Vec<&Transaction> = self.pending_transactions.iter().collect();
        
        let sync_data = serde_json::json!({
            "new_blocks": new_blocks,
            "pending_transactions": pending_txs
        });
        
        sync_data.to_string()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn set_last_sync_block(&mut self, _height: u32) {
        // For WASM, this is a no-op since we don't persist state
        // Native implementation would save this to disk
    }
}

// Native-only methods (not exposed to WASM)
impl Blockchain {
    // Native-only constructor with storage
    #[cfg(feature = "native")]
    pub fn new_with_storage(storage_path: String) -> Self {
        let mut blockchain = Self::new();
        blockchain.storage_path = Some(storage_path);
        blockchain.load_from_disk();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis = Block {
            height: 0,
            hash: "0".repeat(64),
            previous_hash: "0".repeat(64),
            timestamp: Self::current_timestamp(),
            validator: "genesis".to_string(),
            transactions: Vec::new(),
            stake_weight: 0,
            nonce: None,  // WASM-compatible: always None
            data: None,   // WASM-compatible: always None
        };
        self.chain.push(genesis);
    }

    #[cfg(feature = "native")]
    fn mine_block_native(&self, block: &mut Block) {
        // Initialize nonce for native mining if it's None
        if block.nonce.is_none() {
            block.nonce = Some(0);
        }
        
        while !block.hash.starts_with("00") {
            if let Some(ref mut nonce) = block.nonce {
                *nonce += 1;
            }
            block.hash = self.calculate_hash(block);
        }
    }

    fn select_validator(&self) -> Option<String> {
        if self.validators.is_empty() {
            return None;
        }

        let seed = Self::current_timestamp() % self.total_stake.max(1);
        let mut cumulative = 0;

        for (address, validator) in &self.validators {
            if validator.active {
                cumulative += validator.stake;
                if seed < cumulative {
                    return Some(address.clone());
                }
            }
        }

        self.validators.iter()
            .find(|(_, v)| v.active)
            .map(|(addr, _)| addr.clone())
    }

    fn calculate_hash(&self, block: &Block) -> String {
        use sha2::{Sha256, Digest};
        let input = format!("{}{}{}{}", 
            block.height, 
            block.previous_hash, 
            block.validator, 
            block.timestamp
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn current_timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            (js_sys::Date::now() / 1000.0) as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }

    // Enterprise methods (native only)
    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    pub fn add_tenant_blocks(&mut self, update: &TenantBlockchainUpdate) {
        for block in &update.new_blocks {
            self.tenant_blocks.push(block.clone());
        }
        
        if self.tenant_blocks.len() > 100 {
            let start_idx = self.tenant_blocks.len() - 100;
            self.tenant_blocks = self.tenant_blocks[start_idx..].to_vec();
        }
    }

    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    pub fn get_recent_tenant_blocks(&self, limit: usize) -> Vec<serde_json::Value> {
        let start_idx = if self.tenant_blocks.len() > limit {
            self.tenant_blocks.len() - limit
        } else {
            0
        };

        self.tenant_blocks[start_idx..].iter().map(|block| {
            serde_json::json!({
                "network_id": block.network_id,
                "block_id": block.block_id,
                "block_hash": block.block_hash,
                "transactions": block.transactions,
                "timestamp": block.timestamp,
                "previous_hash": block.previous_hash
            })
        }).collect()
    }

    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    #[cfg(feature = "native")]
    pub fn get_tenant_summaries(&self) -> Vec<serde_json::Value> {
        let mut network_stats: std::collections::HashMap<String, (usize, usize, u64)> = std::collections::HashMap::new();

        for block in &self.tenant_blocks {
            let network_id = block.network_id.clone();
            let entry = network_stats.entry(network_id).or_insert((0, 0, 0));
            entry.0 += 1; // block count
            entry.1 += block.transactions.len(); // transaction count
            entry.2 = entry.2.max(block.timestamp); // latest activity
        }

        if network_stats.is_empty() {
            // Fallback for empty state
            vec![serde_json::json!({
                "tenant_id": "no_networks",
                "block_count": 0,
                "transaction_count": 0,
                "last_activity": 0
            })]
        } else {
            network_stats.into_iter().map(|(network_id, (block_count, transaction_count, last_activity))| {
                serde_json::json!({
                    "tenant_id": network_id,
                    "block_count": block_count,
                    "transaction_count": transaction_count,
                    "last_activity": last_activity
                })
            }).collect()
        }
    }

    // Native-only methods that can't be exposed to WASM
    pub fn add_block(&mut self, block: Block) -> bool {
        if self.validate_block(&block) {
            self.chain.push(block);
            #[cfg(feature = "native")]
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        if let Some(last_block) = self.chain.last() {
            block.height == last_block.height + 1 && 
            block.previous_hash == last_block.hash &&
            self.validators.contains_key(&block.validator)
        } else {
            false
        }
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.chain
    }

    pub fn get_latest(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn height(&self) -> u32 {
        (self.chain.len() - 1) as u32
    }

    // Storage methods (native only)
    #[cfg(feature = "native")]
    pub fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            let data = serde_json::json!({
                "chain": self.chain,
                "pending": self.pending_transactions,
                "validators": self.validators,
                "tenant_blocks": self.tenant_blocks,
                "contracts": self.contracts
            });

            if let Ok(json) = serde_json::to_string_pretty(&data) {
                if let Some(parent) = Path::new(path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(path, json);
            }
        }
    }

    #[cfg(feature = "native")]
    pub fn load_from_disk(&mut self) {
        if let Some(path) = &self.storage_path {
            if Path::new(path).exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Ok(chain) = serde_json::from_value(data["chain"].clone()) {
                            self.chain = chain;
                        }
                        if let Ok(pending) = serde_json::from_value(data["pending"].clone()) {
                            self.pending_transactions = pending;
                        }
                        if let Ok(validators) = serde_json::from_value(data["validators"].clone()) {
                            self.validators = validators;
                        }
                        if let Ok(tenant_blocks) = serde_json::from_value(data["tenant_blocks"].clone()) {
                            self.tenant_blocks = tenant_blocks;
                        }
                        if let Ok(contracts) = serde_json::from_value(data["contracts"].clone()) {
                            self.contracts = contracts;
                        }
                    }
                }
            }
        }
    }
}

// Trading OrderBook - shared between WASM and native
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct OrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
    trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub trader: String,
    pub asset: String,
    pub quantity: u64,
    pub price: u64,
    pub side: OrderSide,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub asset: String,
    pub quantity: u64,
    pub price: u64,
    pub buyer: String,
    pub seller: String,
    pub timestamp: u64,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl OrderBook {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            trades: Vec::new(),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn place_buy_order(&mut self, trader: String, asset: String, quantity: u32, price: u32) -> String {
        let order = Order {
            id: format!("buy_{}", self.current_timestamp()),
            trader,
            asset,
            quantity: quantity as u64,  // Convert u32 to u64
            price: price as u64,        // Convert u32 to u64
            side: OrderSide::Buy,
            timestamp: self.current_timestamp(),
        };
        let order_id = order.id.clone();
        self.match_order(order);
        order_id
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn place_sell_order(&mut self, trader: String, asset: String, quantity: u32, price: u32) -> String {
        let order = Order {
            id: format!("sell_{}", self.current_timestamp()),
            trader,
            asset,
            quantity: quantity as u64,  // Convert u32 to u64
            price: price as u64,        // Convert u32 to u64
            side: OrderSide::Sell,
            timestamp: self.current_timestamp(),
        };
        let order_id = order.id.clone();
        self.match_order(order);
        order_id
    }

    fn match_order(&mut self, mut order: Order) {
        let current_time = self.current_timestamp();
        
        let opposite_orders = match order.side {
            OrderSide::Buy => &mut self.asks,
            OrderSide::Sell => &mut self.bids,
        };

        let mut to_remove = Vec::new();
        
        for (i, existing_order) in opposite_orders.iter_mut().enumerate() {
            if existing_order.asset != order.asset {
                continue;
            }

            let can_trade = match order.side {
                OrderSide::Buy => order.price >= existing_order.price,
                OrderSide::Sell => order.price <= existing_order.price,
            };

            if can_trade {
                let trade_quantity = order.quantity.min(existing_order.quantity);
                let trade_price = existing_order.price;

                let trade = Trade {
                    id: format!("trade_{}", current_time),
                    asset: order.asset.clone(),
                    quantity: trade_quantity,
                    price: trade_price,
                    buyer: match order.side {
                        OrderSide::Buy => order.trader.clone(),
                        OrderSide::Sell => existing_order.trader.clone(),
                    },
                    seller: match order.side {
                        OrderSide::Sell => order.trader.clone(),
                        OrderSide::Buy => existing_order.trader.clone(),
                    },
                    timestamp: current_time,
                };

                self.trades.push(trade);
                
                order.quantity -= trade_quantity;
                existing_order.quantity -= trade_quantity;

                if existing_order.quantity == 0 {
                    to_remove.push(i);
                }

                if order.quantity == 0 {
                    break;
                }
            }
        }

        for &index in to_remove.iter().rev() {
            opposite_orders.remove(index);
        }

        if order.quantity > 0 {
            match order.side {
                OrderSide::Buy => {
                    self.bids.push(order);
                    self.bids.sort_by(|a, b| b.price.cmp(&a.price));
                }
                OrderSide::Sell => {
                    self.asks.push(order);
                    self.asks.sort_by(|a, b| a.price.cmp(&b.price));
                }
            }
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_order_book_json(&self) -> String {
        let data = serde_json::json!({
            "bids": self.bids,
            "asks": self.asks
        });
        serde_json::to_string(&data).unwrap_or_default()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_recent_trades_json(&self) -> String {
        let recent_trades: Vec<&Trade> = self.trades.iter().rev().take(20).collect();
        serde_json::to_string(&recent_trades).unwrap_or_default()
    }

    fn current_timestamp(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            (js_sys::Date::now() / 1000.0) as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }
}

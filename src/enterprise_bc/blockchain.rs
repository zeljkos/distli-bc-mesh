// Updated enterprise blockchain - stores tenant blocks directly
use crate::common::{crypto::hash_data, time::current_timestamp};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseBlock {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub validator: String,
    pub transactions: Vec<EnterpriseTransaction>,
    pub merkle_root: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseTransaction {
    pub tx_id: String,
    pub tenant_network: String,
    pub tenant_block_id: u64,
    pub tenant_block_hash: String,
    pub transaction_data: String,
    pub timestamp: u64,
    pub from_peer: String,
    pub contract_address: Option<String>,
    pub gas_used: Option<u64>,
    pub execution_result: Option<String>,
}

// Simple tenant block storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlock {
    pub network_id: String,
    pub block_id: u64,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub previous_hash: String,
    pub from_peer: String,
}

#[derive(Debug, Clone)]
pub struct CrossNetworkOrder {
    pub network_id: String,
    pub block_id: u64,
    pub order_type: String, // "buy" or "sell"
    pub asset: String,
    pub quantity: f64,
    pub price: f64,
    pub peer_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EnterpriseBlockchain {
    pub chain: Vec<EnterpriseBlock>,
    pub pending_transactions: Vec<EnterpriseTransaction>,
    pub tenant_blocks: Vec<TenantBlock>, // NEW: Store tenant blocks directly
    pub validator_id: String,
    pub active_validators: std::collections::HashSet<String>,
    pub last_validator_heartbeat: std::collections::HashMap<String, u64>,
    pub storage_path: String,
    pub tracker_url: Option<String>, // NEW: Track URL for notifications
    // NEW: Track executed cross-network trades to prevent duplicates
    pub executed_cross_network_trades: HashSet<String>, // Set of order identifiers that have been executed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockchainUpdate {
    pub network_id: String,
    pub peer_id: String,
    pub new_blocks: Vec<TenantBlockData>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBlockData {
    pub block_id: u64,
    pub block_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub previous_hash: String,
}

// NEW: Cross-network trade notification
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

impl EnterpriseBlockchain {
    pub fn new(validator_id: String) -> Self {
        let storage_path = format!("data/enterprise_blockchain_{}.json", validator_id);
        let mut blockchain = EnterpriseBlockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            tenant_blocks: Vec::new(), // NEW
            validator_id: validator_id.clone(),
            active_validators: std::collections::HashSet::new(),
            last_validator_heartbeat: std::collections::HashMap::new(),
            storage_path,
            tracker_url: None, // NEW
            executed_cross_network_trades: HashSet::new(), // NEW
        };

        blockchain.load_from_disk();

        if blockchain.chain.is_empty() {
            let genesis = Self::create_genesis_block();
            blockchain.chain.push(genesis);
            blockchain.active_validators.insert(validator_id);
            blockchain.save_to_disk();
        }

        blockchain
    }

    // NEW: Set tracker URL for notifications
    pub fn set_tracker_url(&mut self, url: String) {
        self.tracker_url = Some(url);
    }

    fn generate_order_id(network_id: &str, block_id: u64, order_type: &str, asset: &str, quantity: f64, price: f64) -> String {
        format!("{}:{}:{}:{}:{}:{}", network_id, block_id, order_type, asset, quantity, price)
    }

    fn create_genesis_block() -> EnterpriseBlock {
        EnterpriseBlock {
            height: 0,
            hash: "0".repeat(64),
            previous_hash: "0".repeat(64),
            timestamp: current_timestamp(),
            validator: "genesis".to_string(),
            transactions: Vec::new(),
            merkle_root: "0".repeat(64),
            nonce: 0,
        }
    }

    // NEW: Store tenant block directly (like full blockchain copy)
    pub fn add_tenant_block_directly(
        &mut self,
        network_id: &str,
        block_id: u64,
        block_hash: &str,
        transactions: &[String],
        timestamp: u64,
        previous_hash: &str,
        from_peer: &str
    ) {
        // Check if we already have this block
        let exists = self.tenant_blocks.iter().any(|b| 
            b.network_id == network_id && b.block_id == block_id
        );
        
        if !exists {
            let tenant_block = TenantBlock {
                network_id: network_id.to_string(),
                block_id,
                block_hash: block_hash.to_string(),
                transactions: transactions.to_vec(),
                timestamp,
                previous_hash: previous_hash.to_string(),
                from_peer: from_peer.to_string(),
            };
            
            self.tenant_blocks.push(tenant_block);
            self.save_to_disk();
        }
    }

    // NEW: Cross-network matching functionality
    // UPDATED: Cross-network matching with duplicate prevention
    pub fn extract_and_match_cross_network_orders(&mut self) {
        println!("DEBUG: Starting cross-network matching...");
        println!("DEBUG: Total tenant blocks: {}", self.tenant_blocks.len());
        println!("DEBUG: Already executed trades: {}", self.executed_cross_network_trades.len());

        let orders = self.extract_orders_from_tenant_blocks();
        println!("DEBUG: Extracted {} orders total", orders.len());

        // Filter out already executed orders
        let available_orders: Vec<CrossNetworkOrder> = orders.into_iter()
            .filter(|order| {
                let order_id = Self::generate_order_id(
                    &order.network_id,
                    order.block_id,
                    &order.order_type,
                    &order.asset,
                    order.quantity,
                    order.price
                );
                let is_available = !self.executed_cross_network_trades.contains(&order_id);
                if !is_available {
                    println!("DEBUG: Skipping already executed order: {}", order_id);
                }
                is_available
            })
            .collect();

        println!("DEBUG: Available orders after filtering: {}", available_orders.len());

        // Print all available orders
        for (i, order) in available_orders.iter().enumerate() {
            println!("DEBUG: Available Order {}: {} {} {} @ {} from network {} (block {})",
                     i, order.order_type, order.quantity, order.asset, order.price,
                     order.network_id, order.block_id);
        }

        let matches = self.find_cross_network_matches(&available_orders);
        println!("DEBUG: Found {} potential matches", matches.len());

        if !matches.is_empty() {
            println!("Found {} cross-network matches", matches.len());
            for (buyer_order, seller_order) in matches {
                println!("DEBUG: Executing match: {} {} {} @ {} (buyer: {}) <-> {} {} {} @ {} (seller: {})",
                         buyer_order.order_type, buyer_order.quantity, buyer_order.asset, buyer_order.price, buyer_order.network_id,
                         seller_order.order_type, seller_order.quantity, seller_order.asset, seller_order.price, seller_order.network_id);

                // Mark orders as executed BEFORE executing the trade
                let buyer_order_id = Self::generate_order_id(
                    &buyer_order.network_id,
                    buyer_order.block_id,
                    &buyer_order.order_type,
                    &buyer_order.asset,
                    buyer_order.quantity,
                    buyer_order.price
                );
                let seller_order_id = Self::generate_order_id(
                    &seller_order.network_id,
                    seller_order.block_id,
                    &seller_order.order_type,
                    &seller_order.asset,
                    seller_order.quantity,
                    seller_order.price
                );

                self.executed_cross_network_trades.insert(buyer_order_id.clone());
                self.executed_cross_network_trades.insert(seller_order_id.clone());

                println!("DEBUG: Marked orders as executed: {} and {}", buyer_order_id, seller_order_id);

                self.execute_cross_network_trade(buyer_order, seller_order);
            }

            // Save state after marking orders as executed
            self.save_to_disk();
        } else {
            println!("DEBUG: No cross-network matches found");
        }
    }
    // end of the function 
    fn extract_orders_from_tenant_blocks(&self) -> Vec<CrossNetworkOrder> {
        let mut orders = Vec::new();
        
        // Look at recent tenant blocks (last 50 blocks for efficiency)
        let recent_blocks = if self.tenant_blocks.len() > 50 {
            &self.tenant_blocks[self.tenant_blocks.len() - 50..]
        } else {
            &self.tenant_blocks
        };
        
        for block in recent_blocks {
            for tx in &block.transactions {
                if let Some(order) = self.parse_order_from_transaction(tx, &block.network_id, block.block_id, &block.from_peer, block.timestamp) {
                    orders.push(order);
                }
            }
        }
        
        println!("Extracted {} orders from tenant blocks", orders.len());
        orders
    }
    
    fn parse_order_from_transaction(&self, tx: &str, network_id: &str, block_id: u64, peer_id: &str, timestamp: u64) -> Option<CrossNetworkOrder> {
        println!("DEBUG: Parsing transaction: '{}' from network: {}", tx, network_id);
        
        // Parse transaction strings like:
        // "buy: {"asset":"ETH","quantity":400,"price":100}"
        // "sell: {"asset":"BTC","quantity":1000,"price":2000}"
        
        if let Some(colon_pos) = tx.find(':') {
            let order_type = tx[..colon_pos].trim();
            let json_part = tx[colon_pos + 1..].trim();
            
            println!("DEBUG: Found order_type: '{}', json_part: '{}'", order_type, json_part);
            
            if order_type == "buy" || order_type == "sell" {
                println!("DEBUG: Valid order type, attempting to parse JSON...");
                
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(json_part) {
                    println!("DEBUG: JSON parsed successfully: {}", params);
                    
                    if let (Some(asset), Some(quantity), Some(price)) = (
                        params.get("asset").and_then(|a| a.as_str()),
                        params.get("quantity").and_then(|q| q.as_f64()),
                        params.get("price").and_then(|p| p.as_f64())
                    ) {
                        println!("DEBUG: Successfully extracted - asset: {}, quantity: {}, price: {}", asset, quantity, price);
                        
                        let order = CrossNetworkOrder {
                            network_id: network_id.to_string(),
                            block_id,
                            order_type: order_type.to_string(),
                            asset: asset.to_string(),
                            quantity,
                            price,
                            peer_id: peer_id.to_string(),
                            timestamp,
                        };
                        
                        println!("DEBUG: Created order: {} {} {} @ {} from {}", 
                                 order.order_type, order.quantity, order.asset, order.price, order.network_id);
                        
                        return Some(order);
                    } else {
                        println!("DEBUG: Failed to extract asset/quantity/price from JSON");
                        println!("DEBUG: asset: {:?}", params.get("asset"));
                        println!("DEBUG: quantity: {:?}", params.get("quantity"));
                        println!("DEBUG: price: {:?}", params.get("price"));
                    }
                } else {
                    println!("DEBUG: Failed to parse JSON: {}", json_part);
                }
            } else {
                println!("DEBUG: Not a buy/sell order: '{}'", order_type);
            }
        } else {
            println!("DEBUG: No colon found in transaction: '{}'", tx);
        }
        
        None
    }
    
    fn find_cross_network_matches(&self, orders: &[CrossNetworkOrder]) -> Vec<(CrossNetworkOrder, CrossNetworkOrder)> {
        let mut matches = Vec::new();
        
        for i in 0..orders.len() {
            for j in (i + 1)..orders.len() {
                let order_a = &orders[i];
                let order_b = &orders[j];
                
                // Check if orders can match
                if order_a.network_id != order_b.network_id && // Different networks
                   order_a.asset == order_b.asset && // Same asset
                   order_a.order_type != order_b.order_type && // Buy vs Sell
                   ((order_a.order_type == "buy" && order_a.price >= order_b.price) ||
                    (order_a.order_type == "sell" && order_a.price <= order_b.price)) {
                    
                    println!("Cross-network match found: {} {} {} @ {} ({}) <-> {} {} {} @ {} ({})",
                             order_a.order_type, order_a.quantity, order_a.asset, order_a.price, order_a.network_id,
                             order_b.order_type, order_b.quantity, order_b.asset, order_b.price, order_b.network_id);
                    
                    let (buyer, seller) = if order_a.order_type == "buy" {
                        (order_a.clone(), order_b.clone())
                    } else {
                        (order_b.clone(), order_a.clone())
                    };
                    
                    matches.push((buyer, seller));
                    break; // Only match each order once
                }
            }
        }
        
        matches
    }
    
    fn execute_cross_network_trade(&mut self, buyer_order: CrossNetworkOrder, seller_order: CrossNetworkOrder) {
        let trade_quantity = buyer_order.quantity.min(seller_order.quantity);
        let trade_price = seller_order.price; // Seller's price wins
        
        println!("Executing cross-network trade: {} {} @ {} between {} (buyer) and {} (seller)",
                 trade_quantity, buyer_order.asset, trade_price, buyer_order.network_id, seller_order.network_id);
        
        let trade_id = format!("cross_{}_{}", buyer_order.network_id, seller_order.network_id);
        
        // Create cross-network trade transaction
        let trade_tx = EnterpriseTransaction {
            tx_id: format!("cross_trade_{}_{}", buyer_order.network_id, seller_order.network_id),
            tenant_network: "cross_network".to_string(),
            tenant_block_id: 0,
            tenant_block_hash: "cross_network_trade".to_string(),
            transaction_data: format!("Cross-network trade: {} {} @ {} between {} and {}", 
                                      trade_quantity, buyer_order.asset, trade_price, 
                                      buyer_order.network_id, seller_order.network_id),
            timestamp: current_timestamp(),
            from_peer: "enterprise_matcher".to_string(),
            contract_address: None,
            gas_used: None,
            execution_result: Some(json!({
                "trade_id": trade_id,
                "buyer_network": buyer_order.network_id,
                "seller_network": seller_order.network_id,
                "asset": buyer_order.asset,
                "quantity": trade_quantity,
                "price": trade_price,
                "buyer_order_id": buyer_order.block_id,
                "seller_order_id": seller_order.block_id,
                "timestamp": current_timestamp()
            }).to_string()),
        };
        
        self.pending_transactions.push(trade_tx);
        
        // Auto-mine a block to include the cross-network trade
        if let Some(block) = self.mine_block() {
            self.add_block(block);
            println!("Cross-network trade block mined and added to enterprise blockchain");
        }

        // NEW: Send notification to tracker
        self.notify_tracker_of_cross_network_trade(&buyer_order, &seller_order, trade_quantity, trade_price, &trade_id);
    }

    // NEW: Notify tracker of cross-network trade
    fn notify_tracker_of_cross_network_trade(
        &self,
        buyer_order: &CrossNetworkOrder,
        seller_order: &CrossNetworkOrder,
        quantity: f64,
        price: f64,
        trade_id: &str
    ) {
        if let Some(tracker_url) = &self.tracker_url {
            let notification = CrossNetworkTradeNotification {
                trade_id: trade_id.to_string(),
                buyer_network: buyer_order.network_id.clone(),
                seller_network: seller_order.network_id.clone(),
                asset: buyer_order.asset.clone(),
                quantity,
                price,
                buyer_order_id: buyer_order.block_id,
                seller_order_id: seller_order.block_id,
                timestamp: current_timestamp(),
            };

            // Send async notification to tracker
            let url = format!("{}/api/cross-network-trade", tracker_url);
            let notification_json = serde_json::to_string(&notification).unwrap();
            
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                match client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .body(notification_json)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("Successfully notified tracker of cross-network trade");
                        } else {
                            println!("Failed to notify tracker: HTTP {}", response.status());
                        }
                    }
                    Err(e) => {
                        println!("Error notifying tracker: {}", e);
                    }
                }
            });
        } else {
            println!("No tracker URL configured for notifications");
        }
    }

    pub fn get_recent_cross_network_trades(&self) -> Vec<serde_json::Value> {
        let mut trades = Vec::new();

        println!("DEBUG: Looking for cross-network trades in {} enterprise blocks", self.chain.len());

        // Look through recent enterprise blocks for cross-network trades
        for block in self.chain.iter().rev().take(10) {
            println!("DEBUG: Checking enterprise block {} with {} transactions", block.height, block.transactions.len());

            for tx in &block.transactions {
                if tx.tenant_network == "cross_network" {
                    println!("DEBUG: Found cross-network transaction: {}", tx.transaction_data);

                    if let Some(result) = &tx.execution_result {
                        if let Ok(trade_data) = serde_json::from_str::<serde_json::Value>(result) {
                            trades.push(serde_json::json!({
                                "trade_id": trade_data.get("trade_id").unwrap_or(&serde_json::Value::String("unknown".to_string())),
                                "buyer_network": trade_data.get("buyer_network").unwrap_or(&serde_json::Value::String("unknown".to_string())),
                                "seller_network": trade_data.get("seller_network").unwrap_or(&serde_json::Value::String("unknown".to_string())),
                                "asset": trade_data.get("asset").unwrap_or(&serde_json::Value::String("unknown".to_string())),
                                "quantity": trade_data.get("quantity").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                                "price": trade_data.get("price").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                                "timestamp": trade_data.get("timestamp").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                                "enterprise_block": block.height,
                                "status": "executed",
                                "transaction_data": tx.transaction_data
                            }));

                            println!("DEBUG: Added cross-network trade to results");
                        } else {
                            println!("DEBUG: Failed to parse execution result as JSON: {}", result);
                        }
                    } else {
                        println!("DEBUG: Cross-network transaction has no execution result");
                    }
                }
            }
        }

        println!("DEBUG: Returning {} cross-network trades", trades.len());
        trades
    }

    // NEW: Get recent tenant blocks for display
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
                "previous_hash": block.previous_hash,
                "from_peer": block.from_peer
            })
        }).collect()
    }

    // NEW: Get tenant summaries from stored blocks
    pub fn get_tenant_summaries(&self) -> Vec<serde_json::Value> {
        let mut tenant_stats: std::collections::HashMap<String, (usize, usize, u64, Vec<String>)> = std::collections::HashMap::new();
        
        for block in &self.tenant_blocks {
            let entry = tenant_stats.entry(block.network_id.clone())
                .or_insert((0, 0, 0, Vec::new()));
            
            entry.0 += 1; // block count
            entry.1 += block.transactions.len(); // transaction count
            entry.2 = entry.2.max(block.timestamp); // last activity
            
            // Add recent messages
            for tx in &block.transactions {
                entry.3.push(format!("Block #{}: {}", block.block_id, tx));
                if entry.3.len() > 3 {
                    entry.3.remove(0);
                }
            }
        }
        
        tenant_stats.into_iter().map(|(network_id, (blocks, txs, last_activity, messages))| {
            serde_json::json!({
                "tenant_id": network_id,
                "block_count": blocks,
                "transaction_count": txs,
                "last_activity": last_activity,
                "recent_messages": messages
            })
        }).collect()
    }

    // Keep existing methods for backward compatibility
    pub fn add_tenant_transactions(&mut self, update: TenantBlockchainUpdate) {
        for block in update.new_blocks {
            for (tx_index, tx_data) in block.transactions.iter().enumerate() {
                let enterprise_tx = EnterpriseTransaction {
                    tx_id: format!("{}_{}_{}_{}", update.network_id, block.block_id, tx_index, current_timestamp()),
                    tenant_network: update.network_id.clone(),
                    tenant_block_id: block.block_id,
                    tenant_block_hash: block.block_hash.clone(),
                    transaction_data: tx_data.clone(),
                    timestamp: block.timestamp,
                    from_peer: update.peer_id.clone(),
                    contract_address: None,
                    gas_used: None,
                    execution_result: None,
                };
                
                self.pending_transactions.push(enterprise_tx);
            }
        }
        self.save_to_disk();
    }

    pub fn mine_block(&mut self) -> Option<EnterpriseBlock> {
        if self.pending_transactions.is_empty() {
            return None;
        }

        let last_block = match self.chain.last() {
            Some(block) => block,
            None => return None,
        };
        
        let transactions = self.pending_transactions.clone();
        
        let mut new_block = EnterpriseBlock {
            height: last_block.height + 1,
            hash: String::new(),
            previous_hash: last_block.hash.clone(),
            timestamp: current_timestamp(),
            validator: self.validator_id.clone(),
            transactions: transactions.clone(),
            merkle_root: self.calculate_merkle_root(&transactions),
            nonce: 0,
        };

        self.mine_block_pow(&mut new_block);
        Some(new_block)
    }

    fn mine_block_pow(&self, block: &mut EnterpriseBlock) {
        loop {
            block.hash = self.calculate_block_hash(block);
            if block.hash.starts_with("00") {
                break;
            }
            block.nonce += 1;
        }
    }

    pub fn add_block(&mut self, block: EnterpriseBlock) -> bool {
        if self.validate_block(&block) {
            self.chain.push(block);
            self.pending_transactions.clear();
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    fn calculate_merkle_root(&self, transactions: &[EnterpriseTransaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }
        
        let tx_hashes: Vec<String> = transactions.iter()
            .map(|tx| hash_data(&format!("{}{}{}", tx.tx_id, tx.transaction_data, tx.timestamp)))
            .collect();
        
        hash_data(&tx_hashes.join(""))
    }

    fn calculate_block_hash(&self, block: &EnterpriseBlock) -> String {
        let block_data = format!(
            "{}{}{}{}{}{}",
            block.height,
            block.previous_hash,
            block.timestamp,
            block.validator,
            block.merkle_root,
            block.nonce
        );
        hash_data(&block_data)
    }

    fn validate_block(&self, block: &EnterpriseBlock) -> bool {
        let last_block = match self.chain.last() {
            Some(block) => block,
            None => return false,
        };
        
        if block.height != last_block.height + 1 {
            return false;
        }
        
        if block.previous_hash != last_block.hash {
            return false;
        }
        
        if block.hash != self.calculate_block_hash(block) {
            return false;
        }

        if !block.hash.starts_with("00") {
            return false;
        }
        
        true
    }

    pub fn get_blockchain_info(&self) -> serde_json::Value {
        let total_transactions: usize = self.tenant_blocks.iter()
            .map(|block| block.transactions.len())
            .sum();
        
        let tenant_networks: std::collections::HashSet<String> = self.tenant_blocks.iter()
            .map(|block| block.network_id.clone())
            .collect();
        
        serde_json::json!({
            "height": self.tenant_blocks.len(),
            "validator": self.validator_id,
            "total_blocks": self.tenant_blocks.len(),
            "total_transactions": total_transactions,
            "active_validators": 1,
            "active_tenants": tenant_networks.len(),
            "chain_health": "healthy",
            "validator_status": "online"
        })
    }

    pub fn save_to_disk(&self) {
        let data = serde_json::json!({
            "chain": self.chain,
            "pending_transactions": self.pending_transactions,
            "tenant_blocks": self.tenant_blocks, // NEW
            "validator_id": self.validator_id,
            "active_validators": self.active_validators.iter().cloned().collect::<Vec<_>>(),
            "last_validator_heartbeat": self.last_validator_heartbeat,
            "tracker_url": self.tracker_url, // NEW
            "executed_cross_network_trades": self.executed_cross_network_trades.iter().cloned().collect::<Vec<_>>(), // NEW
        });

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if let Some(parent) = Path::new(&self.storage_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub fn load_from_disk(&mut self) {
        if Path::new(&self.storage_path).exists() {
            if let Ok(content) = fs::read_to_string(&self.storage_path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Ok(chain) = serde_json::from_value(data["chain"].clone()) {
                        self.chain = chain;
                    }
                    if let Ok(pending) = serde_json::from_value(data["pending_transactions"].clone()) {
                        self.pending_transactions = pending;
                    }
                    if let Ok(tenant_blocks) = serde_json::from_value(data["tenant_blocks"].clone()) {
                        self.tenant_blocks = tenant_blocks; // NEW
                    }
                    if let Ok(validators) = serde_json::from_value::<Vec<String>>(data["active_validators"].clone()) {
                        self.active_validators = validators.into_iter().collect();
                    }
                    if let Ok(heartbeat) = serde_json::from_value(data["last_validator_heartbeat"].clone()) {
                        self.last_validator_heartbeat = heartbeat;
                    }
                    if let Ok(tracker_url) = serde_json::from_value(data["tracker_url"].clone()) {
                        self.tracker_url = tracker_url; // NEW
                    }
                    // NEW: Load executed trades
                    if let Ok(executed_trades) = serde_json::from_value::<Vec<String>>(data["executed_cross_network_trades"].clone()) {
                        self.executed_cross_network_trades = executed_trades.into_iter().collect();
                        println!("Loaded {} executed cross-network trades from disk", self.executed_cross_network_trades.len());
                    }
                }
            }
        }
    }

    pub fn update_validator_heartbeat(&mut self, validator_id: String) {
        self.active_validators.insert(validator_id.clone());
        self.last_validator_heartbeat.insert(validator_id, current_timestamp());
        self.save_to_disk();
    }

    pub fn cleanup_stale_validators(&mut self) {
        let current_time = current_timestamp();
        let timeout = 120;

        let stale_validators: Vec<String> = self.last_validator_heartbeat
            .iter()
            .filter(|(_, &heartbeat)| current_time - heartbeat > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for validator_id in stale_validators {
            self.active_validators.remove(&validator_id);
            self.last_validator_heartbeat.remove(&validator_id);
        }
    }
}

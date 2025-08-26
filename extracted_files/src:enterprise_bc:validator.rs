// src/enterprise_bc/validator.rs
use crate::blockchain::{Blockchain, TenantBlockchainUpdate};
use crate::enterprise_bc::api;
use crate::enterprise_bc::order_engine::{EnterpriseOrderEngine, Trade};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct Validator {
    pub id: String,
    pub port: u16,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub order_engine: Arc<RwLock<EnterpriseOrderEngine>>,
    pub stake: u64,
    pub tracker_url: Option<String>,
}

impl Validator {
    pub async fn new(id: String, port: u16, initial_stake: u64) -> Self {
        let mut blockchain = Blockchain::new();
        blockchain.add_validator(id.clone(), initial_stake as u32);
        
        let tracker_url = std::env::var("TRACKER_URL").ok();
        if let Some(ref url) = tracker_url {
            println!("Tracker URL configured: {} (for cross-network trading)", url);
        } else {
            println!("No TRACKER_URL provided - cross-network trades won't be broadcast");
        }
        
        Validator {
            id,
            port,
            blockchain: Arc::new(RwLock::new(blockchain)),
            order_engine: Arc::new(RwLock::new(EnterpriseOrderEngine::new())),
            stake: initial_stake,
            tracker_url,
        }
    }
    
    pub async fn start(self) {
        println!("Starting validator {} with order matching engine", self.id);
        
        let blockchain = self.blockchain.clone();
        let order_engine = self.order_engine.clone();
        let validator_id = self.id.clone();
        
        // Start PoS validation loop
        let validation_blockchain = blockchain.clone();
        let validation_validator_id = validator_id.clone();
        let validation_handle = tokio::spawn(async move {
            Self::pos_validation_loop(validation_blockchain, validation_validator_id).await;
        });
        
        // Start API server
        let api_blockchain = blockchain.clone();
        let api_order_engine = order_engine.clone();
        let api_tracker_url = self.tracker_url.clone();
        let api_handle = tokio::spawn(async move {
            api::start_api_server(self.port, api_blockchain, api_order_engine, api_tracker_url).await;
        });
        
        println!("Enterprise validator ready for cross-network order matching");
        
        tokio::select! {
            _ = validation_handle => println!("Validation stopped"),
            _ = api_handle => println!("API server stopped"),
        }
    }
    
    async fn pos_validation_loop(blockchain: Arc<RwLock<Blockchain>>, validator_id: String) {
        let mut timer = interval(Duration::from_secs(10));
        
        loop {
            timer.tick().await;
            
            {
                let mut bc = blockchain.write().await;
                let pending_count = bc.get_pending_count();
                if pending_count > 0 {
                    if bc.mine_block() {
                        println!("Validator {} created block via PoS", validator_id);
                    }
                }
            }
        }
    }
    
    pub async fn process_tenant_update(&self, update: TenantBlockchainUpdate) -> Vec<Trade> {
        println!("Processing {} blocks from network {}", 
              update.new_blocks.len(), update.network_id);
        
        let mut all_trades = Vec::new();
        
        {
            let mut bc = self.blockchain.write().await;
            bc.add_tenant_blocks(&update);
        }
        
        // Process orders and match them
        {
            let mut engine = self.order_engine.write().await;
            for block in &update.new_blocks {
                let trades = engine.process_block(block);
                all_trades.extend(trades);
            }
        }
        
        // Broadcast matched trades back to networks
        if !all_trades.is_empty() {
            println!("Generated {} cross-network trades", all_trades.len());
            self.broadcast_trades_to_networks(&all_trades).await;
        }
        
        all_trades
    }
    
    async fn broadcast_trades_to_networks(&self, trades: &[Trade]) {
        if let Some(ref tracker_url) = self.tracker_url {
            for trade in trades {
                let trade_notification = serde_json::json!({
                    "type": "cross_network_trade",
                    "trade_id": trade.trade_id,
                    "buyer_network": trade.buyer_network,
                    "seller_network": trade.seller_network,
                    "asset": trade.asset,
                    "quantity": trade.quantity,
                    "price": trade.price,
                    "buyer": trade.buyer,
                    "seller": trade.seller,
                    "timestamp": trade.timestamp
                });
                
                let client = reqwest::Client::new();
                let url = format!("{}/api/cross-network-trade", tracker_url);
                
                match client.post(&url).json(&trade_notification).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("Broadcast trade {} to networks {} and {}", 
                                  trade.trade_id, trade.buyer_network, trade.seller_network);
                        } else {
                            println!("Failed to broadcast trade: HTTP {}", response.status());
                        }
                    }
                    Err(e) => {
                        println!("Failed to broadcast trade: {}", e);
                    }
                }
            }
        }
    }
    
    pub async fn get_order_book_status(&self) -> serde_json::Value {
        let engine = self.order_engine.read().await;
        let order_book_summary = engine.get_order_book_summary();
        let recent_trades = engine.get_recent_trades(10);
        
        serde_json::json!({
            "order_book": order_book_summary,
            "recent_trades": recent_trades,
            "total_recent_trades": recent_trades.len()
        })
    }
}

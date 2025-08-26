// src/enterprise_bc/order_engine.rs - FIXED BORROWING ISSUE
use serde::{Deserialize, Serialize};
use crate::blockchain::{TenantBlockData, Transaction, TransactionType};
use std::collections::HashSet;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub order_id: String,
    pub trader: String,
    pub network_id: String,
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
    pub trade_id: String,
    pub asset: String,
    pub quantity: u64,
    pub price: u64,
    pub buyer: String,
    pub seller: String,
    pub buyer_network: String,
    pub seller_network: String,
    pub timestamp: u64,
}

pub struct EnterpriseOrderEngine {
    pub buy_orders: Vec<OrderBookEntry>,
    pub sell_orders: Vec<OrderBookEntry>, 
    pub recent_trades: Vec<Trade>,
    pub processed_transactions: HashSet<String>, // Track processed transaction IDs

}

impl EnterpriseOrderEngine {
    pub fn new() -> Self {
        Self {
            buy_orders: Vec::new(),
            sell_orders: Vec::new(),
            recent_trades: Vec::new(),
            processed_transactions: HashSet::new(),
        }
    }

    pub fn process_block(&mut self, block: &TenantBlockData) -> Vec<Trade> {
        println!("Processing block from network {} with {} transactions", 
                 block.network_id, block.transactions.len());
        
        let mut new_trades = Vec::new();
        
        for tx_string in &block.transactions {
            if let Ok(tx) = serde_json::from_str::<Transaction>(tx_string) {
                // Skip if already processed
                if self.processed_transactions.contains(&tx.id) {
                    println!("Skipping already processed transaction: {}", tx.id);
                    continue;
                }
                
                if let TransactionType::Trading { asset, quantity, price } = &tx.tx_type {
                    println!("Processing new trading transaction: {} {} {} @ {}", 
                             tx.id, asset, quantity, price);
                    
                    let order_side = if tx.id.contains("buy_") { 
                        OrderSide::Buy 
                    } else { 
                        OrderSide::Sell 
                    };
                    
                    let order = OrderBookEntry {
                        order_id: tx.id.clone(),
                        trader: tx.from.clone(),
                        network_id: block.network_id.clone(),
                        asset: asset.clone(),
                        quantity: *quantity,
                        price: *price,
                        side: order_side,
                        timestamp: tx.timestamp,
                    };
                    
                    let trades = self.process_order(order);
                    new_trades.extend(trades);
                    
                    // Mark as processed
                    self.processed_transactions.insert(tx.id.clone());
                }
            }
        }
        
        println!("Block processing complete. Generated {} trades", new_trades.len());
        new_trades
    }


    fn process_order(&mut self, order: OrderBookEntry) -> Vec<Trade> {
        println!("Processing order: {:?} {} {} @ {} from {}", 
                 order.side, order.quantity, order.asset, order.price, order.network_id);
        
        let mut trades = Vec::new();
        let mut remaining_order = order;
        
        // Get opposite side orders count for logging
        let opposite_count = match remaining_order.side {
            OrderSide::Buy => self.sell_orders.len(),
            OrderSide::Sell => self.buy_orders.len(),
        };
        
        println!("Looking for matches against {} opposite orders", opposite_count);
        
        // Process matches by taking ownership of the opposite orders temporarily
        let mut opposite_orders = match remaining_order.side {
            OrderSide::Buy => std::mem::take(&mut self.sell_orders),
            OrderSide::Sell => std::mem::take(&mut self.buy_orders),
        };
        
        let mut i = 0;
        while i < opposite_orders.len() && remaining_order.quantity > 0 {
            let opposite_order = &mut opposite_orders[i];
            
            // Check if orders can match (same asset and compatible price)
            let can_match = remaining_order.asset == opposite_order.asset && 
                           Self::prices_match(&remaining_order, opposite_order);
            
            println!("Checking match with order {} {} @ {}: can_match = {}", 
                     opposite_order.quantity, opposite_order.asset, opposite_order.price, can_match);
            
            if can_match {
                let trade_quantity = remaining_order.quantity.min(opposite_order.quantity);
                let trade_price = opposite_order.price; // Maker's price wins
                
                let trade = Trade {
                    trade_id: format!("trade_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()),
                    asset: remaining_order.asset.clone(),
                    quantity: trade_quantity,
                    price: trade_price,
                    buyer: match remaining_order.side {
                        OrderSide::Buy => remaining_order.trader.clone(),
                        OrderSide::Sell => opposite_order.trader.clone(),
                    },
                    seller: match remaining_order.side {
                        OrderSide::Sell => remaining_order.trader.clone(),
                        OrderSide::Buy => opposite_order.trader.clone(),
                    },
                    buyer_network: match remaining_order.side {
                        OrderSide::Buy => remaining_order.network_id.clone(),
                        OrderSide::Sell => opposite_order.network_id.clone(),
                    },
                    seller_network: match remaining_order.side {
                        OrderSide::Sell => remaining_order.network_id.clone(),
                        OrderSide::Buy => opposite_order.network_id.clone(),
                    },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                
                println!("TRADE EXECUTED: {} {} {} @ {} between networks {} and {}", 
                         trade.trade_id, trade.quantity, trade.asset, trade.price,
                         trade.buyer_network, trade.seller_network);
                
                trades.push(trade);
                
                // Update quantities
                remaining_order.quantity -= trade_quantity;
                opposite_order.quantity -= trade_quantity;
                
                // Remove fully filled opposite order
                if opposite_order.quantity == 0 {
                    println!("Removing fully filled opposite order");
                    opposite_orders.remove(i);
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        // Put the opposite orders back
        match remaining_order.side {
            OrderSide::Buy => self.sell_orders = opposite_orders,
            OrderSide::Sell => self.buy_orders = opposite_orders,
        }
        
        // Add remaining quantity to order book
        if remaining_order.quantity > 0 {
            println!("Adding remaining order to book: {} {}", remaining_order.quantity, remaining_order.asset);
            match remaining_order.side {
                OrderSide::Buy => {
                    self.buy_orders.push(remaining_order);
                    self.buy_orders.sort_by(|a, b| b.price.cmp(&a.price)); // Highest price first
                }
                OrderSide::Sell => {
                    self.sell_orders.push(remaining_order);
                    self.sell_orders.sort_by(|a, b| a.price.cmp(&b.price)); // Lowest price first
                }
            }
        }
        
        // Store trades
        self.recent_trades.extend(trades.clone());
        if self.recent_trades.len() > 100 {
            let start = self.recent_trades.len() - 100;
            self.recent_trades = self.recent_trades[start..].to_vec();
        }
        
        println!("Order processing complete. Current state:");
        println!("Buy orders: {}, Sell orders: {}, Total trades: {}", 
                 self.buy_orders.len(), self.sell_orders.len(), self.recent_trades.len());
        
        trades
    }
    
    // FIXED: Made this a static function to avoid borrow checker issues
    fn prices_match(order1: &OrderBookEntry, order2: &OrderBookEntry) -> bool {
        match (&order1.side, &order2.side) {
            (OrderSide::Buy, OrderSide::Sell) => order1.price >= order2.price,
            (OrderSide::Sell, OrderSide::Buy) => order1.price <= order2.price,
            _ => false,
        }
    }

    pub fn get_order_book_summary(&self) -> serde_json::Value {
        let mut asset_summary = std::collections::HashMap::new();
        
        for order in &self.buy_orders {
            let entry = asset_summary.entry(order.asset.clone()).or_insert_with(|| serde_json::json!({
                "bids": 0,
                "asks": 0,
                "total_orders": 0
            }));
            entry["bids"] = serde_json::Value::Number(serde_json::Number::from(
                entry["bids"].as_u64().unwrap_or(0) + 1
            ));
            entry["total_orders"] = serde_json::Value::Number(serde_json::Number::from(
                entry["total_orders"].as_u64().unwrap_or(0) + 1
            ));
        }
        
        for order in &self.sell_orders {
            let entry = asset_summary.entry(order.asset.clone()).or_insert_with(|| serde_json::json!({
                "bids": 0,
                "asks": 0,
                "total_orders": 0
            }));
            entry["asks"] = serde_json::Value::Number(serde_json::Number::from(
                entry["asks"].as_u64().unwrap_or(0) + 1
            ));
            entry["total_orders"] = serde_json::Value::Number(serde_json::Number::from(
                entry["total_orders"].as_u64().unwrap_or(0) + 1
            ));
        }
        
        serde_json::Value::Object(asset_summary.into_iter().collect())
    }

    pub fn get_recent_trades(&self, limit: usize) -> &[Trade] {
        let start = if self.recent_trades.len() > limit {
            self.recent_trades.len() - limit
        } else {
            0
        };
        &self.recent_trades[start..]
    }

    pub fn get_all_orders(&self) -> serde_json::Value {
        // Map orders to include all fields explicitly
        let buy_orders_with_network: Vec<serde_json::Value> = self.buy_orders.iter().map(|o| {
            serde_json::json!({
                "order_id": o.order_id,
                "trader": o.trader,
                "network_id": o.network_id,  // Explicitly include
                "asset": o.asset,
                "quantity": o.quantity,
                "price": o.price,
                "side": "buy",
                "timestamp": o.timestamp
            })
        }).collect();

        let sell_orders_with_network: Vec<serde_json::Value> = self.sell_orders.iter().map(|o| {
            serde_json::json!({
                "order_id": o.order_id,
                "trader": o.trader,
                "network_id": o.network_id,  // Explicitly include
                "asset": o.asset,
                "quantity": o.quantity,
                "price": o.price,
                "side": "sell",
                "timestamp": o.timestamp
            })
        }).collect();

        serde_json::json!({
            "buy_orders": buy_orders_with_network,
            "sell_orders": sell_orders_with_network
        })
    }
    
}

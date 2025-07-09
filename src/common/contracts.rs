// src/common/contracts.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: String,
    pub name: String,
    pub code: String, // Contract logic identifier
    pub state: serde_json::Value,
    pub owner: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_id: String,
    pub function: String,
    pub params: serde_json::Value,
    pub caller: String,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub gas_used: u64,
    pub state_changes: Option<serde_json::Value>,
    pub events: Vec<ContractEvent>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

// Simple VM for contract execution
pub struct ContractVM {
    contracts: HashMap<String, SmartContract>,
}

impl ContractVM {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    pub fn deploy_contract(&mut self, contract: SmartContract) -> Result<String, String> {
        let id = contract.id.clone();
        self.contracts.insert(id.clone(), contract);
        Ok(id)
    }

    pub fn call_contract(&mut self, call: ContractCall) -> ContractResult {
        let contract = match self.contracts.get_mut(&call.contract_id) {
            Some(c) => c,
            None => return ContractResult {
                success: false,
                result: serde_json::Value::Null,
                gas_used: 1,
                state_changes: None,
                events: vec![],
                error: Some("Contract not found".to_string()),
            }
        };

        // Execute based on contract type
        match contract.code.as_str() {
            "trading" => Self::execute_trading_contract(contract, call),
            _ => ContractResult {
                success: false,
                result: serde_json::Value::Null,
                gas_used: 1,
                state_changes: None,
                events: vec![],
                error: Some("Unknown contract type".to_string()),
            }
        }
    }

    fn execute_trading_contract(contract: &mut SmartContract, call: ContractCall) -> ContractResult {
        let mut state = contract.state.clone();
        let mut events = vec![];
        
        // Initialize state if empty
        if state.is_null() {
            state = serde_json::json!({
                "orderBook": {
                    "bids": [],
                    "asks": []
                },
                "trades": [],
                "nextOrderId": 1
            });
        }

        let result = match call.function.as_str() {
            "buy" => Self::handle_buy_order(&mut state, &call, &mut events),
            "sell" => Self::handle_sell_order(&mut state, &call, &mut events),
            "cancel" => Self::handle_cancel_order(&mut state, &call, &mut events),
            "getOrderBook" => Self::handle_get_order_book(&state, &call),
            "getTrades" => Self::handle_get_trades(&state, &call),
            _ => return ContractResult {
                success: false,
                result: serde_json::Value::Null,
                gas_used: 1,
                state_changes: None,
                events: vec![],
                error: Some("Unknown function".to_string()),
            }
        };

        // Update contract state
        contract.state = state.clone();

        ContractResult {
            success: true,
            result,
            gas_used: 10, // Simple gas model
            state_changes: Some(state),
            events,
            error: None,
        }
    }

    fn handle_buy_order(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let asset = params["asset"].as_str().unwrap_or("");
        let quantity = params["quantity"].as_f64().unwrap_or(0.0);
        let price = params["price"].as_f64().unwrap_or(0.0);

        if asset.is_empty() || quantity <= 0.0 || price <= 0.0 {
            return serde_json::json!({
                "error": "Invalid parameters: asset, quantity, and price must be valid"
            });
        }

        let order_id = state["nextOrderId"].as_u64().unwrap_or(1);
        state["nextOrderId"] = serde_json::Value::Number(serde_json::Number::from(order_id + 1));

        let mut order = serde_json::json!({
            "id": order_id,
            "type": "buy",
            "asset": asset,
            "quantity": quantity,
            "price": price,
            "trader": call.caller,
            "timestamp": crate::common::time::current_timestamp()
        });

        // Try to match with existing sell orders
        let trades = Self::match_orders(&mut order, &mut state["orderBook"]["asks"], events);

        // Add remaining quantity to bids if any left
        if order["quantity"].as_f64().unwrap_or(0.0) > 0.0 {
            state["orderBook"]["bids"].as_array_mut().unwrap().push(order.clone());
            // Sort bids by price (highest first)
            Self::sort_bids(&mut state["orderBook"]["bids"]);
        }

        events.push(ContractEvent {
            event_type: "OrderPlaced".to_string(),
            data: serde_json::json!({
                "orderId": order_id,
                "type": "buy",
                "asset": asset,
                "quantity": quantity,
                "price": price,
                "trader": call.caller
            }),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "orderId": order_id,
            "trades": trades,
            "message": format!("Buy order placed: {} {} @ {}", quantity, asset, price)
        })
    }

    fn handle_sell_order(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let asset = params["asset"].as_str().unwrap_or("");
        let quantity = params["quantity"].as_f64().unwrap_or(0.0);
        let price = params["price"].as_f64().unwrap_or(0.0);

        if asset.is_empty() || quantity <= 0.0 || price <= 0.0 {
            return serde_json::json!({
                "error": "Invalid parameters: asset, quantity, and price must be valid"
            });
        }

        let order_id = state["nextOrderId"].as_u64().unwrap_or(1);
        state["nextOrderId"] = serde_json::Value::Number(serde_json::Number::from(order_id + 1));

        let mut order = serde_json::json!({
            "id": order_id,
            "type": "sell",
            "asset": asset,
            "quantity": quantity,
            "price": price,
            "trader": call.caller,
            "timestamp": crate::common::time::current_timestamp()
        });

        // Try to match with existing buy orders
        let trades = Self::match_orders(&mut order, &mut state["orderBook"]["bids"], events);

        // Add remaining quantity to asks if any left
        if order["quantity"].as_f64().unwrap_or(0.0) > 0.0 {
            state["orderBook"]["asks"].as_array_mut().unwrap().push(order.clone());
            // Sort asks by price (lowest first)
            Self::sort_asks(&mut state["orderBook"]["asks"]);
        }

        events.push(ContractEvent {
            event_type: "OrderPlaced".to_string(),
            data: serde_json::json!({
                "orderId": order_id,
                "type": "sell",
                "asset": asset,
                "quantity": quantity,
                "price": price,
                "trader": call.caller
            }),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "orderId": order_id,
            "trades": trades,
            "message": format!("Sell order placed: {} {} @ {}", quantity, asset, price)
        })
    }

    fn handle_cancel_order(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let order_id = call.params["orderId"].as_u64().unwrap_or(0);
        
        if order_id == 0 {
            return serde_json::json!({
                "error": "Invalid order ID"
            });
        }

        // Try to remove from bids
        let bids = state["orderBook"]["bids"].as_array_mut().unwrap();
        if let Some(pos) = bids.iter().position(|o| 
            o["id"].as_u64() == Some(order_id) && 
            o["trader"].as_str() == Some(&call.caller)
        ) {
            let order = bids.remove(pos);
            events.push(ContractEvent {
                event_type: "OrderCancelled".to_string(),
                data: order.clone(),
                timestamp: crate::common::time::current_timestamp(),
            });
            return serde_json::json!({
                "message": format!("Buy order cancelled: {} {} @ {}", 
                    order["quantity"], order["asset"], order["price"])
            });
        }

        // Try to remove from asks
        let asks = state["orderBook"]["asks"].as_array_mut().unwrap();
        if let Some(pos) = asks.iter().position(|o| 
            o["id"].as_u64() == Some(order_id) && 
            o["trader"].as_str() == Some(&call.caller)
        ) {
            let order = asks.remove(pos);
            events.push(ContractEvent {
                event_type: "OrderCancelled".to_string(),
                data: order.clone(),
                timestamp: crate::common::time::current_timestamp(),
            });
            return serde_json::json!({
                "message": format!("Sell order cancelled: {} {} @ {}", 
                    order["quantity"], order["asset"], order["price"])
            });
        }

        serde_json::json!({
            "error": "Order not found or not owned by caller"
        })
    }

    fn handle_get_order_book(state: &serde_json::Value, call: &ContractCall) -> serde_json::Value {
        let asset_filter = call.params["asset"].as_str();
        
        let mut bids = state["orderBook"]["bids"].as_array().unwrap_or(&vec![]).clone();
        let mut asks = state["orderBook"]["asks"].as_array().unwrap_or(&vec![]).clone();
        
        if let Some(asset) = asset_filter {
            bids.retain(|order| order["asset"].as_str() == Some(asset));
            asks.retain(|order| order["asset"].as_str() == Some(asset));
        }

        serde_json::json!({
            "bids": bids,
            "asks": asks,
            "asset": asset_filter
        })
    }

    fn handle_get_trades(state: &serde_json::Value, call: &ContractCall) -> serde_json::Value {
        let limit = call.params["limit"].as_u64().unwrap_or(50) as usize;
        let asset_filter = call.params["asset"].as_str();
        
        let mut trades = state["trades"].as_array().unwrap_or(&vec![]).clone();
        
        if let Some(asset) = asset_filter {
            trades.retain(|trade| trade["asset"].as_str() == Some(asset));
        }
        
        // Return most recent trades
        if trades.len() > limit {
            trades = trades[trades.len() - limit..].to_vec();
        }
        
        serde_json::json!({
            "trades": trades,
            "count": trades.len()
        })
    }

    fn match_orders(order: &mut serde_json::Value, opposite_orders: &mut serde_json::Value, events: &mut Vec<ContractEvent>) -> Vec<serde_json::Value> {
        let mut trades = vec![];
        let order_type = order["type"].as_str().unwrap();
        let order_price = order["price"].as_f64().unwrap();
        let mut remaining_quantity = order["quantity"].as_f64().unwrap();
        
        let orders_array = opposite_orders.as_array_mut().unwrap();
        let mut i = 0;
        
        while i < orders_array.len() && remaining_quantity > 0.0 {
            let opposite_order = &mut orders_array[i];
            let opposite_price = opposite_order["price"].as_f64().unwrap();
            let opposite_quantity = opposite_order["quantity"].as_f64().unwrap();
            
            // Check if prices match
            let can_trade = match order_type {
                "buy" => order_price >= opposite_price,  // Buy at or above ask price
                "sell" => order_price <= opposite_price, // Sell at or below bid price
                _ => false,
            };
            
            if can_trade {
                let trade_quantity = remaining_quantity.min(opposite_quantity);
                let trade_price = opposite_price; // Price discovery: taker pays maker's price
                
                // Create trade record
                let trade = serde_json::json!({
                    "id": format!("{}_{}", crate::common::time::current_timestamp(), trades.len()),
                    "asset": order["asset"],
                    "quantity": trade_quantity,
                    "price": trade_price,
                    "buyer": if order_type == "buy" { order["trader"].clone() } else { opposite_order["trader"].clone() },
                    "seller": if order_type == "sell" { order["trader"].clone() } else { opposite_order["trader"].clone() },
                    "timestamp": crate::common::time::current_timestamp()
                });
                
                trades.push(trade.clone());
                
                // Emit trade event
                events.push(ContractEvent {
                    event_type: "Trade".to_string(),
                    data: trade,
                    timestamp: crate::common::time::current_timestamp(),
                });
                
                // Update quantities
                remaining_quantity -= trade_quantity;
                opposite_order["quantity"] = serde_json::Value::Number(
                    serde_json::Number::from_f64(opposite_quantity - trade_quantity).unwrap()
                );
                
                // Remove opposite order if fully filled
                if opposite_order["quantity"].as_f64().unwrap() <= 0.0 {
                    orders_array.remove(i);
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        // Update remaining quantity in original order
        order["quantity"] = serde_json::Value::Number(
            serde_json::Number::from_f64(remaining_quantity).unwrap()
        );
        
        trades
    }

    fn sort_bids(bids: &mut serde_json::Value) {
        if let Some(array) = bids.as_array_mut() {
            array.sort_by(|a, b| {
                let price_a = a["price"].as_f64().unwrap_or(0.0);
                let price_b = b["price"].as_f64().unwrap_or(0.0);
                price_b.partial_cmp(&price_a).unwrap() // Highest price first
            });
        }
    }

    fn sort_asks(asks: &mut serde_json::Value) {
        if let Some(array) = asks.as_array_mut() {
            array.sort_by(|a, b| {
                let price_a = a["price"].as_f64().unwrap_or(0.0);
                let price_b = b["price"].as_f64().unwrap_or(0.0);
                price_a.partial_cmp(&price_b).unwrap() // Lowest price first
            });
        }
    }

    pub fn get_contract(&self, contract_id: &str) -> Option<&SmartContract> {
        self.contracts.get(contract_id)
    }

    pub fn list_contracts(&self) -> Vec<&SmartContract> {
        self.contracts.values().collect()
    }
}

// Helper function to create trading contract
pub fn create_trading_contract(owner: String) -> SmartContract {
    SmartContract {
        id: "trading_contract".to_string(),
        name: "Simple Trading Contract".to_string(),
        code: "trading".to_string(),
        state: serde_json::json!({
            "orderBook": {
                "bids": [],
                "asks": []
            },
            "trades": [],
            "nextOrderId": 1
        }),
        owner,
        created_at: crate::common::time::current_timestamp(),
    }
}

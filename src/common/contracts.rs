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
            "gsm_roaming" => Self::execute_gsm_roaming_contract(contract, call),
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

    fn execute_gsm_roaming_contract(contract: &mut SmartContract, call: ContractCall) -> ContractResult {
        let mut state = contract.state.clone();
        let mut events = vec![];
        
        // Initialize state if empty
        if state.is_null() {
            state = serde_json::json!({
                "activeSessions": {},
                "billingHistory": [],
                "networkRates": {},
                "subscribers": {}
            });
        }

        let result = match call.function.as_str() {
            "connect" => Self::handle_roaming_connect(&mut state, &call, &mut events),
            "disconnect" => Self::handle_roaming_disconnect(&mut state, &call, &mut events),
            "processMinuteBilling" => Self::handle_minute_billing(&mut state, &call, &mut events),
            "setRate" => Self::handle_set_rate(&mut state, &call, &mut events),
            "getSession" => Self::handle_get_session(&state, &call),
            "getBillingHistory" => Self::handle_get_billing_history(&state, &call),
            "getActiveSessions" => Self::handle_get_active_sessions(&state, &call),
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
            gas_used: 10,
            state_changes: Some(state),
            events,
            error: None,
        }
    }

    fn handle_roaming_connect(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let imsi = params["imsi"].as_str().unwrap_or("");
        let home_network = params["homeNetwork"].as_str().unwrap_or("");
        let visiting_network = params["visitingNetwork"].as_str().unwrap_or("");
        let antenna_id = params["antennaId"].as_str().unwrap_or("");
        let guest_wallet = params["guestWallet"].as_str().unwrap_or("");
        let host_wallet = params["hostWallet"].as_str().unwrap_or("");

        if imsi.is_empty() || home_network.is_empty() || visiting_network.is_empty() {
            return serde_json::json!({
                "error": "Invalid parameters: IMSI, home network, and visiting network are required"
            });
        }

        // Generate session ID
        let session_id = format!("{}_{}", imsi, crate::common::time::current_timestamp());
        
        // Get rate for this network pair (default 10 if not set)
        let rate_key = format!("{}_{}", home_network, visiting_network);
        let rate_per_minute = state["networkRates"][&rate_key].as_u64().unwrap_or(10);

        // Create session
        let session = serde_json::json!({
            "sessionId": session_id,
            "imsi": imsi,
            "homeNetwork": home_network,
            "visitingNetwork": visiting_network,
            "antennaId": antenna_id,
            "guestWallet": guest_wallet,
            "hostWallet": host_wallet,
            "ratePerMinute": rate_per_minute,
            "startTime": crate::common::time::current_timestamp(),
            "minutesBilled": 0,
            "totalCost": 0,
            "active": true
        });

        // Store session
        state["activeSessions"][&session_id] = session.clone();

        events.push(ContractEvent {
            event_type: "RoamingConnected".to_string(),
            data: serde_json::json!({
                "sessionId": session_id,
                "imsi": imsi,
                "homeNetwork": home_network,
                "visitingNetwork": visiting_network,
                "antennaId": antenna_id,
                "ratePerMinute": rate_per_minute
            }),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "sessionId": session_id,
            "message": format!("GSM roaming session started for IMSI {} on antenna {}", imsi, antenna_id),
            "ratePerMinute": rate_per_minute
        })
    }

    fn handle_roaming_disconnect(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let session_id = params["sessionId"].as_str().unwrap_or("");

        if session_id.is_empty() {
            return serde_json::json!({
                "error": "Session ID is required"
            });
        }

        let session = match state["activeSessions"][session_id].clone() {
            serde_json::Value::Null => {
                return serde_json::json!({
                    "error": "Session not found"
                });
            }
            session => session
        };

        // Calculate total duration and cost
        let start_time = session["startTime"].as_u64().unwrap_or(0);
        let end_time = crate::common::time::current_timestamp();
        let duration_minutes = ((end_time - start_time) / 60000).max(1); // At least 1 minute
        let rate_per_minute = session["ratePerMinute"].as_u64().unwrap_or(10);
        let total_cost = duration_minutes * rate_per_minute;

        // Create billing record
        let billing_record = serde_json::json!({
            "sessionId": session_id,
            "imsi": session["imsi"],
            "homeNetwork": session["homeNetwork"],
            "visitingNetwork": session["visitingNetwork"],
            "guestWallet": session["guestWallet"],
            "hostWallet": session["hostWallet"],
            "startTime": start_time,
            "endTime": end_time,
            "durationMinutes": duration_minutes,
            "ratePerMinute": rate_per_minute,
            "totalCost": total_cost,
            "disconnectTime": end_time
        });

        // Add to billing history
        state["billingHistory"].as_array_mut().unwrap_or(&mut vec![]).push(billing_record.clone());

        // Remove from active sessions
        state["activeSessions"][session_id] = serde_json::Value::Null;

        events.push(ContractEvent {
            event_type: "RoamingDisconnected".to_string(),
            data: billing_record.clone(),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "message": format!("GSM roaming session {} ended", session_id),
            "durationMinutes": duration_minutes,
            "totalCost": total_cost,
            "billingRecord": billing_record
        })
    }

    fn handle_minute_billing(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let session_id = params["sessionId"].as_str().unwrap_or("");

        if session_id.is_empty() {
            return serde_json::json!({
                "error": "Session ID is required"
            });
        }

        let mut session = match state["activeSessions"][session_id].clone() {
            serde_json::Value::Null => {
                return serde_json::json!({
                    "error": "Session not found or not active"
                });
            }
            session => session
        };

        let rate_per_minute = session["ratePerMinute"].as_u64().unwrap_or(10);
        let minutes_billed = session["minutesBilled"].as_u64().unwrap_or(0);
        let new_minute_number = minutes_billed + 1;

        // Update session
        session["minutesBilled"] = serde_json::Value::Number(serde_json::Number::from(new_minute_number));
        session["totalCost"] = serde_json::Value::Number(serde_json::Number::from(new_minute_number * rate_per_minute));
        
        // Store updated session
        state["activeSessions"][session_id] = session.clone();

        events.push(ContractEvent {
            event_type: "MinuteBilled".to_string(),
            data: serde_json::json!({
                "sessionId": session_id,
                "imsi": session["imsi"],
                "minuteNumber": new_minute_number,
                "amount": rate_per_minute,
                "totalCost": new_minute_number * rate_per_minute,
                "guestWallet": session["guestWallet"],
                "hostWallet": session["hostWallet"]
            }),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "sessionId": session_id,
            "minuteNumber": new_minute_number,
            "amount": rate_per_minute,
            "totalCost": new_minute_number * rate_per_minute,
            "message": format!("Minute {} billed for session {}", new_minute_number, session_id)
        })
    }

    fn handle_set_rate(state: &mut serde_json::Value, call: &ContractCall, events: &mut Vec<ContractEvent>) -> serde_json::Value {
        let params = &call.params;
        let home_network = params["homeNetwork"].as_str().unwrap_or("");
        let visiting_network = params["visitingNetwork"].as_str().unwrap_or("");
        let rate_per_minute = params["ratePerMinute"].as_u64().unwrap_or(0);

        if home_network.is_empty() || visiting_network.is_empty() || rate_per_minute == 0 {
            return serde_json::json!({
                "error": "Home network, visiting network, and rate per minute are required"
            });
        }

        let rate_key = format!("{}_{}", home_network, visiting_network);
        state["networkRates"][&rate_key] = serde_json::Value::Number(serde_json::Number::from(rate_per_minute));

        events.push(ContractEvent {
            event_type: "RateSet".to_string(),
            data: serde_json::json!({
                "homeNetwork": home_network,
                "visitingNetwork": visiting_network,
                "ratePerMinute": rate_per_minute
            }),
            timestamp: crate::common::time::current_timestamp(),
        });

        serde_json::json!({
            "message": format!("Rate set: {} -> {} at {} per minute", home_network, visiting_network, rate_per_minute)
        })
    }

    fn handle_get_session(state: &serde_json::Value, call: &ContractCall) -> serde_json::Value {
        let session_id = call.params["sessionId"].as_str().unwrap_or("");
        
        if session_id.is_empty() {
            return serde_json::json!({
                "error": "Session ID is required"
            });
        }

        match &state["activeSessions"][session_id] {
            serde_json::Value::Null => serde_json::json!({
                "error": "Session not found"
            }),
            session => session.clone()
        }
    }

    fn handle_get_billing_history(state: &serde_json::Value, call: &ContractCall) -> serde_json::Value {
        let limit = call.params["limit"].as_u64().unwrap_or(10) as usize;
        let imsi_filter = call.params["imsi"].as_str();
        
        let mut history = state["billingHistory"].as_array().unwrap_or(&vec![]).clone();
        
        if let Some(imsi) = imsi_filter {
            history.retain(|record| record["imsi"].as_str() == Some(imsi));
        }
        
        // Return most recent records
        if history.len() > limit {
            history = history[history.len() - limit..].to_vec();
        }
        
        serde_json::json!({
            "billingHistory": history,
            "count": history.len()
        })
    }

    fn handle_get_active_sessions(state: &serde_json::Value, _call: &ContractCall) -> serde_json::Value {
        let active_sessions: Vec<_> = state["activeSessions"]
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .values()
            .filter(|v| !v.is_null())
            .cloned()
            .collect();
        
        serde_json::json!({
            "activeSessions": active_sessions,
            "count": active_sessions.len()
        })
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

// Helper function to create GSM roaming contract
pub fn create_gsm_roaming_contract(owner: String, contract_id: Option<String>) -> SmartContract {
    SmartContract {
        id: contract_id.unwrap_or_else(|| format!("gsm_roaming_{}", crate::common::time::current_timestamp())),
        name: "GSM Roaming Contract".to_string(),
        code: "gsm_roaming".to_string(),
        state: serde_json::json!({
            "activeSessions": {},
            "billingHistory": [],
            "networkRates": {
                "vodafone_tmobile": 15,
                "tmobile_vodafone": 12,
                "orange_verizon": 20,
                "verizon_orange": 18
            },
            "subscribers": {}
        }),
        owner,
        created_at: crate::common::time::current_timestamp(),
    }
}

// examples/trading_test.rs
// Example: How to test the trading functionality

use distli_mesh_bc::common::{
    Blockchain, 
    Transaction,
    ContractCall,
    create_trading_contract
};

fn main() {
    println!("🚀 Testing Smart Contract Trading System");
    
    // Create blockchain with trading contract
    let mut blockchain = Blockchain::new();
    
    println!("✅ Blockchain created with trading contract");
    println!("📋 Available contracts: {}", blockchain.list_contracts().len());
    
    // Test 1: Place a buy order
    println!("\n📈 Test 1: Placing buy order");
    let buy_call = ContractCall {
        contract_id: "trading_contract".to_string(),
        function: "buy".to_string(),
        params: serde_json::json!({
            "asset": "BTC",
            "quantity": 1.0,
            "price": 50000.0
        }),
        caller: "alice".to_string(),
        gas_limit: 100,
    };
    
    let buy_tx = blockchain.call_contract(buy_call, "alice".to_string());
    
    if let Transaction::ContractCall { result, .. } = &buy_tx {
        if let Some(res) = result {
            if res.success {
                println!("✅ Buy order placed successfully");
                if let Some(msg) = res.result.get("message") {
                    println!("   📝 {}", msg.as_str().unwrap_or(""));
                }
            } else {
                println!("❌ Buy order failed: {:?}", res.error);
            }
        }
    }
    
    // Test 2: Place a sell order (should match)
    println!("\n📉 Test 2: Placing sell order");
    let sell_call = ContractCall {
        contract_id: "trading_contract".to_string(),
        function: "sell".to_string(),
        params: serde_json::json!({
            "asset": "BTC",
            "quantity": 0.5,
            "price": 49000.0  // Lower than Alice's bid, should match
        }),
        caller: "bob".to_string(),
        gas_limit: 100,
    };
    
    let sell_tx = blockchain.call_contract(sell_call, "bob".to_string());
    
    if let Transaction::ContractCall { result, .. } = &sell_tx {
        if let Some(res) = result {
            if res.success {
                println!("✅ Sell order placed successfully");
                if let Some(msg) = res.result.get("message") {
                    println!("   📝 {}", msg.as_str().unwrap_or(""));
                }
                
                // Check if trades occurred
                if let Some(trades) = res.result.get("trades") {
                    if let Some(trades_array) = trades.as_array() {
                        if !trades_array.is_empty() {
                            println!("   💰 Trade executed!");
                            for trade in trades_array {
                                println!("      🔄 {} {} @ {} - {} -> {}", 
                                    trade["quantity"], 
                                    trade["asset"],
                                    trade["price"],
                                    trade["seller"].as_str().unwrap_or("unknown"),
                                    trade["buyer"].as_str().unwrap_or("unknown")
                                );
                            }
                        }
                    }
                }
            } else {
                println!("❌ Sell order failed: {:?}", res.error);
            }
        }
    }
    
    // Test 3: Check order book
    println!("\n📊 Test 3: Checking order book");
    let orderbook = blockchain.get_order_book(Some("BTC"));
    
    if let Some(bids) = orderbook.get("bids") {
        if let Some(bids_array) = bids.as_array() {
            println!("   📈 Buy orders: {}", bids_array.len());
            for bid in bids_array {
                println!("      ${} for {} {} by {}", 
                    bid["price"], 
                    bid["quantity"], 
                    bid["asset"],
                    bid["trader"].as_str().unwrap_or("unknown")
                );
            }
        }
    }
    
    if let Some(asks) = orderbook.get("asks") {
        if let Some(asks_array) = asks.as_array() {
            println!("   📉 Sell orders: {}", asks_array.len());
            for ask in asks_array {
                println!("      ${} for {} {} by {}", 
                    ask["price"], 
                    ask["quantity"], 
                    ask["asset"],
                    ask["trader"].as_str().unwrap_or("unknown")
                );
            }
        }
    }
    
    // Test 4: Check recent trades
    println!("\n💹 Test 4: Checking recent trades");
    let trades = blockchain.get_recent_trades(Some("BTC"), Some(5));
    
    if let Some(trades_data) = trades.get("trades") {
        if let Some(trades_array) = trades_data.as_array() {
            println!("   📈 Recent trades: {}", trades_array.len());
            for trade in trades_array {
                println!("      💰 {} {} @ ${} - {} -> {}", 
                    trade["quantity"], 
                    trade["asset"],
                    trade["price"],
                    trade["seller"].as_str().unwrap_or("unknown"),
                    trade["buyer"].as_str().unwrap_or("unknown")
                );
            }
        }
    }
    
    // Test 5: Mine a block with transactions
    println!("\n⛏️  Test 5: Mining block with transactions");
    blockchain.add_message("Hello blockchain world!".to_string(), "charlie".to_string());
    
    if let Some(block) = blockchain.mine_pending_block() {
        println!("✅ Block #{} mined with {} transactions", block.id, block.transactions.len());
        println!("   📝 Block data: {}", block.data);
        
        for (i, tx) in block.transactions.iter().enumerate() {
            match tx {
                Transaction::Message { sender, data, .. } => {
                    println!("      {}. 💬 {}: {}", i + 1, sender, data);
                },
                Transaction::ContractCall { sender, call, result, .. } => {
                    let status = if result.as_ref().map_or(false, |r| r.success) { "✅" } else { "❌" };
                    println!("      {}. {} {}: {}()", i + 1, status, sender, call.function);
                },
                Transaction::ContractDeploy { sender, contract, .. } => {
                    println!("      {}. 📄 {}: Deployed {}", i + 1, sender, contract.name);
                }
            }
        }
    }
    
    println!("\n🎉 Trading system test completed!");
    println!("📊 Final stats:");
    println!("   - Blockchain height: {}", blockchain.height());
    println!("   - Total blocks: {}", blockchain.chain.len());
    println!("   - Active contracts: {}", blockchain.list_contracts().len());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_trading() {
        let mut blockchain = Blockchain::new();
        
        // Place buy order
        let buy_call = ContractCall {
            contract_id: "trading_contract".to_string(),
            function: "buy".to_string(),
            params: serde_json::json!({
                "asset": "ETH",
                "quantity": 10.0,
                "price": 3000.0
            }),
            caller: "alice".to_string(),
            gas_limit: 100,
        };
        
        let tx = blockchain.call_contract(buy_call, "alice".to_string());
        
        // Verify transaction was created
        match tx {
            Transaction::ContractCall { result, .. } => {
                assert!(result.is_some());
                let res = result.unwrap();
                assert!(res.success);
            },
            _ => panic!("Expected contract call transaction")
        }
    }
    
    #[test]
    fn test_trade_matching() {
        let mut blockchain = Blockchain::new();
        
        // Alice places buy order
        let buy_call = ContractCall {
            contract_id: "trading_contract".to_string(),
            function: "buy".to_string(),
            params: serde_json::json!({
                "asset": "ETH",
                "quantity": 10.0,
                "price": 3000.0
            }),
            caller: "alice".to_string(),
            gas_limit: 100,
        };
        blockchain.call_contract(buy_call, "alice".to_string());
        
        // Bob places sell order (should match)
        let sell_call = ContractCall {
            contract_id: "trading_contract".to_string(),
            function: "sell".to_string(),
            params: serde_json::json!({
                "asset": "ETH",
                "quantity": 5.0,
                "price": 2950.0  // Lower than Alice's bid, should match
            }),
            caller: "bob".to_string(),
            gas_limit: 100,
        };
        let tx = blockchain.call_contract(sell_call, "bob".to_string());
        
        // Check that trade occurred
        if let Transaction::ContractCall { result, .. } = tx {
            let res = result.unwrap();
            assert!(res.success);
            
            // Should have trades in the result
            let trades = res.result["trades"].as_array().unwrap();
            assert!(trades.len() > 0);
            
            println!("Trade executed: {}", serde_json::to_string_pretty(&trades[0]).unwrap());
        }
        
        // Check recent trades
        let recent_trades = blockchain.get_recent_trades(Some("ETH"), Some(5));
        let trades_array = recent_trades["trades"].as_array().unwrap();
        assert!(trades_array.len() > 0);
    }
}

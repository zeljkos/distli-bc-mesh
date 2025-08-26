// Example: Testing smart contracts
// Run with: cargo run --example test_smart_contracts --features native

use distli_mesh_bc::blockchain::SmartContractExecutor;

fn main() {
    println!("=== Smart Contract System Test ===\n");
    
    let mut executor = SmartContractExecutor::new();
    let owner = "alice".to_string();
    
    // Test 1: Counter Contract
    println!("1. COUNTER CONTRACT:");
    println!("--------------------");
    let deploy_result = executor.deploy_contract(
        "counter".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Deploy: {}", deploy_result);
    
    // Call increment
    let result = executor.call_contract_with_gas(
        "contract_1".to_string(),
        "increment".to_string(),
        "{}".to_string(),
        owner.clone(),
        100
    );
    println!("Increment: {}", result);
    
    // Get count
    let result = executor.call_contract(
        "contract_1".to_string(),
        "get_count".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Get count: {}", result);
    
    // Test 2: Order Book  
    println!("\n2. ORDER BOOK CONTRACT:");
    println!("----------------------");
    let deploy_result = executor.deploy_contract(
        "orderbook".to_string(),
        r#"{"asset": "BTC/USD"}"#.to_string(),
        owner.clone()
    );
    println!("Deploy: {}", deploy_result);
    
    // Place orders
    let result = executor.call_contract(
        "contract_2".to_string(),
        "place_bid".to_string(),
        r#"{"price": 45000, "quantity": 1.5}"#.to_string(),
        owner.clone()
    );
    println!("Place bid: {}", result);
    
    let result = executor.call_contract(
        "contract_2".to_string(),
        "place_ask".to_string(),
        r#"{"price": 46000, "quantity": 1.0}"#.to_string(),
        "bob".to_string()
    );
    println!("Place ask: {}", result);
    
    // Get orderbook
    let result = executor.call_contract(
        "contract_2".to_string(),
        "get_orderbook".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Orderbook: {}", result);
    
    // Test 3: NFT Contract
    println!("\n3. NFT CONTRACT:");
    println!("----------------");
    let deploy_result = executor.deploy_contract(
        "nft".to_string(),
        r#"{"name": "CryptoArt", "symbol": "CART"}"#.to_string(),
        owner.clone()
    );
    println!("Deploy: {}", deploy_result);
    
    // Mint NFT
    let result = executor.call_contract(
        "contract_3".to_string(),
        "mint".to_string(),
        r#"{"metadata": "Unique digital artwork #1"}"#.to_string(),
        owner.clone()
    );
    println!("Mint: {}", result);
    
    // Get NFT metadata
    let result = executor.call_contract(
        "contract_3".to_string(),
        "get_metadata".to_string(),
        r#"{"token_id": 1}"#.to_string(),
        owner.clone()
    );
    println!("Metadata: {}", result);
    
    // Test 4: Gas Metering
    println!("\n4. GAS METERING TEST:");
    println!("--------------------");
    
    // Normal gas limit
    let result = executor.call_contract_with_gas(
        "contract_1".to_string(),
        "increment".to_string(),
        "{}".to_string(),
        owner.clone(),
        50
    );
    println!("Normal gas (50): {}", result);
    
    // Very low gas limit
    let result = executor.call_contract_with_gas(
        "contract_1".to_string(),
        "increment".to_string(),
        "{}".to_string(),
        owner.clone(),
        1
    );
    println!("Low gas (1): {}", result);
    
    // List all contracts
    println!("\n5. ALL DEPLOYED CONTRACTS:");
    println!("--------------------------");
    let contracts = executor.list_contracts();
    println!("{}", contracts);
    
    // Get contract states
    println!("\n6. CONTRACT STATES:");
    println!("-------------------");
    for i in 1..=3 {
        let state = executor.get_contract_state(format!("contract_{}", i));
        println!("Contract {}: {}", i, state);
    }
}
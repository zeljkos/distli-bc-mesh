// Simple test for smart contracts
use distli_mesh_bc::blockchain::SmartContractExecutor;

fn main() {
    println!("Testing Smart Contract System");
    
    let mut executor = SmartContractExecutor::new();
    let owner = "user123".to_string();
    
    // Test Counter Contract
    println!("\n1. Testing Counter Contract:");
    let counter_result = executor.deploy_contract(
        "counter".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Deploy counter: {}", counter_result);
    
    let counter_id = "contract_1".to_string();
    
    // Increment
    let result = executor.call_contract_with_gas(
        counter_id.clone(),
        "increment".to_string(),
        "{}".to_string(),
        owner.clone(),
        100
    );
    println!("Increment: {}", result);
    
    // Get count
    let result = executor.call_contract(
        counter_id.clone(),
        "get_count".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Get count: {}", result);
    
    // Test Order Book Contract
    println!("\n2. Testing Order Book Contract:");
    let orderbook_result = executor.deploy_contract(
        "orderbook".to_string(),
        r#"{"asset": "BTC/USD"}"#.to_string(),
        owner.clone()
    );
    println!("Deploy orderbook: {}", orderbook_result);
    
    let orderbook_id = "contract_2".to_string();
    
    // Place bid
    let result = executor.call_contract(
        orderbook_id.clone(),
        "place_bid".to_string(),
        r#"{"price": 50000.0, "quantity": 0.5}"#.to_string(),
        owner.clone()
    );
    println!("Place bid: {}", result);
    
    // Place ask
    let result = executor.call_contract(
        orderbook_id.clone(),
        "place_ask".to_string(),
        r#"{"price": 51000.0, "quantity": 0.3}"#.to_string(),
        "user456".to_string()
    );
    println!("Place ask: {}", result);
    
    // Get orderbook
    let result = executor.call_contract(
        orderbook_id.clone(),
        "get_orderbook".to_string(),
        "{}".to_string(),
        owner.clone()
    );
    println!("Get orderbook: {}", result);
    
    // Test NFT Contract
    println!("\n3. Testing NFT Contract:");
    let nft_result = executor.deploy_contract(
        "nft".to_string(),
        r#"{"name": "TestNFT", "symbol": "TNFT"}"#.to_string(),
        owner.clone()
    );
    println!("Deploy NFT: {}", nft_result);
    
    let nft_id = "contract_3".to_string();
    
    // Mint NFT
    let result = executor.call_contract(
        nft_id.clone(),
        "mint".to_string(),
        r#"{"metadata": "First NFT", "to": "user789"}"#.to_string(),
        owner.clone()
    );
    println!("Mint NFT: {}", result);
    
    // Get owner
    let result = executor.call_contract(
        nft_id.clone(),
        "get_owner".to_string(),
        r#"{"token_id": 1}"#.to_string(),
        owner.clone()
    );
    println!("Get NFT owner: {}", result);
    
    // Test gas limits
    println!("\n4. Testing Gas Limits:");
    let result = executor.call_contract_with_gas(
        counter_id.clone(),
        "increment".to_string(),
        "{}".to_string(),
        owner.clone(),
        1  // Very low gas limit
    );
    println!("Low gas call result: {}", result);
    
    // List all contracts
    println!("\n5. All Contracts:");
    let contracts = executor.list_contracts();
    println!("Contracts: {}", contracts);
}
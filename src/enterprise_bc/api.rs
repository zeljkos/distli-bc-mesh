// src/enterprise_bc/api.rs - SIMPLIFIED WORKING VERSION
use crate::blockchain::{Blockchain, TenantBlockchainUpdate};
use crate::enterprise_bc::order_engine::{EnterpriseOrderEngine, Trade};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

pub async fn start_api_server(
    port: u16, 
    blockchain: Arc<RwLock<Blockchain>>,
    order_engine: Arc<RwLock<EnterpriseOrderEngine>>,
    tracker_url: Option<String>
) {
    println!("Starting Enterprise API server with order matching on port {}", port);

    let blockchain_filter = warp::any().map(move || blockchain.clone());
    let order_engine_filter = warp::any().map(move || order_engine.clone());
    let tracker_filter = warp::any().map(move || tracker_url.clone());

    // Main endpoint for processing tenant blockchain updates
    let tenant_blockchain_update = warp::path("api")
        .and(warp::path("tenant-blockchain-update"))
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and(order_engine_filter.clone())
        .and(tracker_filter.clone())
        .and_then(handle_tenant_blockchain_update);

    // Order book status endpoint
    let order_book_status = warp::path("api")
        .and(warp::path("order-book-status"))
        .and(warp::get())
        .and(order_engine_filter.clone())
        .and_then(handle_order_book_status);

    // Debug endpoint to see all orders
    let debug_orders = warp::path("api")
        .and(warp::path("debug-orders"))
        .and(warp::get())
        .and(order_engine_filter.clone())
        .and_then(handle_debug_orders);

    // Existing endpoints
    let status = warp::path("api")
        .and(warp::path("status"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_status);

    let blocks = warp::path("api")
        .and(warp::path("blocks"))
        .and(warp::get())
        .and(warp::query::<BlocksQuery>())
        .and(blockchain_filter.clone())
        .and_then(handle_blocks);

    let tenants = warp::path("api")
        .and(warp::path("tenants"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_tenants);

    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        })));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let routes = tenant_blockchain_update
        .or(order_book_status)
        .or(debug_orders)
        .or(status)
        .or(blocks)
        .or(tenants)
        .or(health)
        .with(cors);

    println!("Enterprise API server ready on http://0.0.0.0:{}", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

#[derive(serde::Deserialize)]
struct BlocksQuery {
    limit: Option<usize>,
}

// Updated handle_tenant_blockchain_update in src/enterprise_bc/api.rs
// Updated handle_tenant_blockchain_update in src/enterprise_bc/api.rs
async fn handle_tenant_blockchain_update(
    update: TenantBlockchainUpdate,
    blockchain: Arc<RwLock<Blockchain>>,
    order_engine: Arc<RwLock<EnterpriseOrderEngine>>,
    tracker_url: Option<String>
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("=== ENTERPRISE BC: Processing tenant update from network: {} ===", update.network_id);
    
    // Store blocks and process orders (existing code)
    let mut all_trades = Vec::new();
    {
        let mut bc = blockchain.write().await;
        bc.add_tenant_blocks(&update);
        
        let mut engine = order_engine.write().await;
        for block in &update.new_blocks {
            let block_trades = engine.process_block(block);
            all_trades.extend(block_trades);
        }
    }
    
    // NEW: Create and send execution blocks back to edge networks
    if !all_trades.is_empty() {
        println!("Broadcasting {} cross-network trades back to edge", all_trades.len());
        
        if let Some(ref tracker_url) = tracker_url {
            for trade in &all_trades {
                // Send to both buyer and seller networks
                send_execution_block_to_network(trade, &trade.buyer_network, tracker_url).await;
                if trade.buyer_network != trade.seller_network {
                    send_execution_block_to_network(trade, &trade.seller_network, tracker_url).await;
                }
            }
        }
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "trades_executed": all_trades.len()
    })))
}

// NEW: Send execution block to specific network
async fn send_execution_block_to_network(trade: &Trade, network_id: &str, tracker_url: &str) {
    // Create execution transaction in enterprise format
    let execution_tx = serde_json::json!({
        "id": format!("exec_{}", trade.trade_id),
        "from": trade.buyer,
        "to": trade.seller,
        "amount": trade.quantity * trade.price / 100,
        "tx_type": {
            "TradeExecution": {
                "asset": trade.asset,
                "quantity": trade.quantity,
                "price": trade.price,
                "buyer": trade.buyer,
                "seller": trade.seller,
                "trade_id": trade.trade_id
            }
        },
        "timestamp": trade.timestamp
    });
    
    // Create execution block
    let execution_block = serde_json::json!({
        "height": 999999, // Special height for enterprise execution blocks
        "hash": format!("exec_{}", trade.trade_id),
        "previous_hash": "enterprise",
        "timestamp": trade.timestamp,
        "validator": "enterprise_bc",
        "transactions": [execution_tx],
        "stake_weight": 1000
    });
    
    // Send to tracker for forwarding to network
    let notification = serde_json::json!({
        "type": "enterprise_execution_block",
        "network_id": network_id,
        "execution_block": execution_block,
        "trade": {
            "trade_id": trade.trade_id,
            "asset": trade.asset,
            "quantity": trade.quantity,
            "price": trade.price,
            "buyer": trade.buyer,
            "seller": trade.seller,
            "buyer_network": trade.buyer_network,
            "seller_network": trade.seller_network,
            "timestamp": trade.timestamp
        }
    });
    
    let client = reqwest::Client::new();
    let url = format!("{}/api/enterprise-execution", tracker_url);
    
    match client.post(&url).json(&notification).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("Sent execution block for trade {} to network {}", trade.trade_id, network_id);
            } else {
                println!("Failed to send execution block: HTTP {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to send execution block: {}", e);
        }
    }
}
////////

async fn send_trade_to_tracker(trade: &crate::enterprise_bc::order_engine::Trade, tracker_url: &str) {
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
    
    println!("Sending trade notification to tracker: {}", url);
    
    match client.post(&url).json(&trade_notification).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("Successfully sent trade {} to tracker", trade.trade_id);
            } else {
                println!("Failed to send trade to tracker: HTTP {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to send trade to tracker: {}", e);
        }
    }
}

async fn handle_order_book_status(
    order_engine: Arc<RwLock<EnterpriseOrderEngine>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let engine = order_engine.read().await;
    
    let order_book_summary = engine.get_order_book_summary();
    let recent_trades = engine.get_recent_trades(20);
    
    println!("Order book status requested - {} recent trades", recent_trades.len());
    
    let response = serde_json::json!({
        "order_book": order_book_summary,
        "recent_trades": recent_trades,
        "total_recent_trades": recent_trades.len(),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    Ok(warp::reply::json(&response))
}

async fn handle_debug_orders(
    order_engine: Arc<RwLock<EnterpriseOrderEngine>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let engine = order_engine.read().await;
    let all_orders = engine.get_all_orders();
    
    println!("Debug orders requested");
    
    Ok(warp::reply::json(&all_orders))
}

async fn handle_status(
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    
    let tenant_summaries = bc.get_tenant_summaries();
    let total_tenant_blocks = bc.get_recent_tenant_blocks(1000).len();
    let pending_count = bc.get_pending_count();
    let validator_count = bc.get_validator_count();
    
    let status = serde_json::json!({
        "height": total_tenant_blocks,
        "validator": "enterprise_validator",
        "total_blocks": total_tenant_blocks,
        "total_transactions": pending_count,
        "active_validators": validator_count,
        "active_tenants": tenant_summaries.len(),
        "chain_health": "healthy",
        "consensus": "proof_of_stake"
    });
    
    Ok(warp::reply::json(&status))
}

async fn handle_blocks(
    query: BlocksQuery,
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let limit = query.limit.unwrap_or(20);
    let bc = blockchain.read().await;
    let blocks = bc.get_recent_tenant_blocks(limit);
    
    Ok(warp::reply::json(&blocks))
}

async fn handle_tenants(
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let tenants = bc.get_tenant_summaries();
    
    Ok(warp::reply::json(&serde_json::json!({
        "tenants": tenants,
        "total_tenants": tenants.len()
    })))
}

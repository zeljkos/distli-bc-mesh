// Updated api.rs - handles full transaction data with CORS and missing endpoints
use crate::enterprise_bc::{EnterpriseBlockchain, TenantBlockchainUpdate};
use crate::common::api_utils;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde_json::json;
use tracing::info;

pub async fn start_api_server(
    port: u16,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) {
    info!("Starting Enterprise API server on port {}", port);

    let blockchain_filter = warp::any().map(move || blockchain.clone());

    // Handle full tenant blockchain updates (not summaries)
    let tenant_blockchain_update = warp::path("api")
        .and(warp::path("tenant-blockchain-update"))
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(handle_tenant_blockchain_update);

    // GET /api/status - Get blockchain status
    let status = warp::path("api")
        .and(warp::path("status"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_status);

    // GET /api/blocks - Get recent blocks with full transaction data
    let blocks = warp::path("api")
        .and(warp::path("blocks"))
        .and(warp::get())
        .and(warp::query::<BlocksQuery>())
        .and(blockchain_filter.clone())
        .and_then(handle_blocks);

    // GET /api/transactions - Get all transactions (for smart contracts)
    let transactions = warp::path("api")
        .and(warp::path("transactions"))
        .and(warp::get())
        .and(warp::query::<TransactionsQuery>())
        .and(blockchain_filter.clone())
        .and_then(handle_transactions);

    // NEW: GET /api/tenants - Get tenant summaries (required by dashboard)
    let tenants = warp::path("api")
        .and(warp::path("tenants"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_tenants);

    // GET /api/network-transactions/:network_id - Get transactions by network
    let network_transactions = warp::path("api")
        .and(warp::path("network-transactions"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_network_transactions);

    // Health check
    let health = warp::path("health")
        .and(warp::get())
        .map(|| api_utils::health_check());

    // CORS configuration - Allow dashboard to access API
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let routes = tenant_blockchain_update
        .or(status)
        .or(blocks)
        .or(transactions)
        .or(tenants)
        .or(network_transactions)
        .or(health)
        .with(cors);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

#[derive(serde::Deserialize)]
struct BlocksQuery {
    limit: Option<usize>,
}

#[derive(serde::Deserialize)]
struct TransactionsQuery {
    limit: Option<usize>,
    network: Option<String>,
}

// Handle full tenant blockchain updates immediately
async fn handle_tenant_blockchain_update(
    update: TenantBlockchainUpdate,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received full blockchain update from network: {}", update.network_id);
    
    let blocks_count = update.new_blocks.len();
    let mut transactions_count = 0;
    
    // Count transactions in all blocks
    for block in &update.new_blocks {
        transactions_count += block.transactions.len();
    }
    
    info!("Processing {} blocks with {} total transactions", blocks_count, transactions_count);
    
    // Process immediately - create enterprise block
    let result = {
        let mut bc = blockchain.write().await;
        
        // Add all tenant transactions to pending pool
        bc.add_tenant_transactions(update.clone());
        
        // Mine block immediately
        if let Some(new_block) = bc.mine_block() {
            info!("Mined enterprise block {} with {} transactions", 
                  new_block.height, new_block.transactions.len());
            
            let success = bc.add_block(new_block.clone());
            if success {
                Some(new_block)
            } else {
                None
            }
        } else {
            None
        }
    };
    
    match result {
        Some(block) => {
            info!("✅ Created enterprise block {} immediately for tenant network {}", 
                  block.height, update.network_id);
            
            Ok(warp::reply::json(&json!({
                "status": "success",
                "message": "Tenant blockchain processed and enterprise block created",
                "network_id": update.network_id,
                "enterprise_block_height": block.height,
                "tenant_blocks_processed": blocks_count,
                "transactions_processed": transactions_count,
                "immediate_processing": true
            })))
        }
        None => {
            Ok(warp::reply::json(&json!({
                "status": "error",
                "message": "Failed to create enterprise block",
                "network_id": update.network_id
            })))
        }
    }
}

async fn handle_status(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let status = bc.get_blockchain_info();
    Ok(warp::reply::json(&status))
}

// ENHANCED: Return blocks with full transaction data
async fn handle_blocks(
    query: BlocksQuery,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let limit = query.limit.unwrap_or(10);
    let bc = blockchain.read().await;

    let start_idx = if bc.chain.len() > limit {
        bc.chain.len() - limit
    } else {
        0
    };

    let blocks: Vec<serde_json::Value> = bc.chain[start_idx..].iter().map(|block| {
        json!({
            "height": block.height,
            "hash": block.hash,
            "previous_hash": block.previous_hash,
            "timestamp": block.timestamp,
            "validator": block.validator,
            "transaction_count": block.transactions.len(),
            "transactions": block.transactions.iter().map(|tx| json!({
                "tx_id": tx.tx_id,
                "tenant_network": tx.tenant_network,
                "tenant_block_id": tx.tenant_block_id,
                "transaction_data": tx.transaction_data,
                "timestamp": tx.timestamp,
                "from_peer": tx.from_peer
            })).collect::<Vec<_>>(),
            "merkle_root": block.merkle_root,
            "nonce": block.nonce
        })
    }).collect();

    Ok(warp::reply::json(&blocks))
}

// NEW: Handle tenants endpoint (required by dashboard)
async fn handle_tenants(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    
    // Extract tenant summaries from blockchain data
    let mut tenant_networks = std::collections::HashMap::new();
    
    // Analyze all transactions to build tenant summaries
    for block in &bc.chain {
        for tx in &block.transactions {
            let entry = tenant_networks.entry(tx.tenant_network.clone()).or_insert_with(|| {
                json!({
                    "tenant_id": tx.tenant_network,
                    "block_count": 0,
                    "transaction_count": 0,
                    "last_activity": tx.timestamp,
                    "recent_messages": Vec::<String>::new()
                })
            });
            
            // Update counters
            if let Some(block_count) = entry.get_mut("block_count") {
                *block_count = json!(block_count.as_u64().unwrap_or(0) + 1);
            }
            if let Some(tx_count) = entry.get_mut("transaction_count") {
                *tx_count = json!(tx_count.as_u64().unwrap_or(0) + 1);
            }
            
            // Update last activity
            if let Some(last_activity) = entry.get_mut("last_activity") {
                if tx.timestamp > last_activity.as_u64().unwrap_or(0) {
                    *last_activity = json!(tx.timestamp);
                }
            }
            
            // Add recent message
            if let Some(messages) = entry.get_mut("recent_messages") {
                if let Some(messages_array) = messages.as_array_mut() {
                    messages_array.push(json!(format!("Block #{}: {}", tx.tenant_block_id, tx.transaction_data)));
                    // Keep only last 3 messages
                    if messages_array.len() > 3 {
                        messages_array.remove(0);
                    }
                }
            }
        }
    }
    
    let tenants: Vec<serde_json::Value> = tenant_networks.into_values().collect();
    
    Ok(warp::reply::json(&json!({
        "tenants": tenants,
        "total_tenants": tenants.len()
    })))
}

// Get all transactions for smart contract execution
async fn handle_transactions(
    query: TransactionsQuery,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let limit = query.limit.unwrap_or(100);
    
    let all_transactions = if let Some(network) = query.network {
        bc.get_transactions_by_network(&network)
    } else {
        bc.get_all_transactions()
    };
    
    let transactions: Vec<serde_json::Value> = all_transactions
        .iter()
        .rev()
        .take(limit)
        .map(|tx| json!({
            "tx_id": tx.tx_id,
            "tenant_network": tx.tenant_network,
            "tenant_block_id": tx.tenant_block_id,
            "transaction_data": tx.transaction_data,
            "timestamp": tx.timestamp,
            "from_peer": tx.from_peer,
            "contract_address": tx.contract_address,
            "gas_used": tx.gas_used,
            "execution_result": tx.execution_result
        }))
        .collect();
    
    Ok(warp::reply::json(&json!({
        "total_transactions": all_transactions.len(),
        "returned_transactions": transactions.len(),
        "transactions": transactions
    })))
}

// Get transactions by specific tenant network
async fn handle_network_transactions(
    network_id: String,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let network_transactions = bc.get_transactions_by_network(&network_id);
    
    let transactions: Vec<serde_json::Value> = network_transactions
        .iter()
        .map(|tx| json!({
            "tx_id": tx.tx_id,
            "tenant_block_id": tx.tenant_block_id,
            "transaction_data": tx.transaction_data,
            "timestamp": tx.timestamp,
            "from_peer": tx.from_peer
        }))
        .collect();
    
    Ok(warp::reply::json(&json!({
        "network_id": network_id,
        "transaction_count": transactions.len(),
        "transactions": transactions
    })))
}

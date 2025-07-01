// Enhanced enterprise_bc/api.rs - Simple debugging version
use crate::enterprise_bc::{EnterpriseBlockchain, TenantBlockchainUpdate};
use crate::common::api_utils;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde_json::json;
use tracing::{info, warn}; // Removed unused 'error' import

pub async fn start_api_server(
    port: u16,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) {
    info!("Starting Enterprise API server on port {}", port);

    let blockchain_filter = warp::any().map(move || blockchain.clone());

    let tenant_blockchain_update = warp::path("api")
        .and(warp::path("tenant-blockchain-update"))
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(handle_tenant_blockchain_update);

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
        .map(|| api_utils::health_check());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let routes = tenant_blockchain_update
        .or(status)
        .or(blocks)
        .or(tenants)
        .or(health)
        .with(cors);

    info!("Enterprise API server ready on http://0.0.0.0:{}", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

#[derive(serde::Deserialize)]
struct BlocksQuery {
    limit: Option<usize>,
}

async fn handle_tenant_blockchain_update(
    update: TenantBlockchainUpdate,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received tenant blockchain update from network: {}", update.network_id);
    info!("Update contains {} blocks", update.new_blocks.len());
    
    let blocks_count = update.new_blocks.len();
    let mut transactions_count = 0;
    
    // Log each block being stored
    for (i, block) in update.new_blocks.iter().enumerate() {
        info!("Block {}: id={}, transactions={}", i, block.block_id, block.transactions.len());
        transactions_count += block.transactions.len();
    }
    
    {
        let mut bc = blockchain.write().await;
        
        // Store each tenant block
        for block in &update.new_blocks {
            bc.add_tenant_block_directly(
                &update.network_id,
                block.block_id,
                &block.block_hash,
                &block.transactions,
                block.timestamp,
                &block.previous_hash,
                &update.peer_id
            );
        }
        
        info!("Total tenant blocks now: {}", bc.tenant_blocks.len());
    }
    
    info!("Stored {} blocks with {} transactions from {}", 
          blocks_count, transactions_count, update.network_id);
    
    Ok(warp::reply::json(&json!({
        "status": "success",
        "message": "Tenant blocks stored",
        "network_id": update.network_id,
        "blocks_stored": blocks_count,
        "transactions_stored": transactions_count
    })))
}

async fn handle_status(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let status = bc.get_blockchain_info();
    info!("Status request - tenant blocks: {}", bc.tenant_blocks.len());
    Ok(warp::reply::json(&status))
}

async fn handle_blocks(
    query: BlocksQuery,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let limit = query.limit.unwrap_or(20);
    info!("Blocks request with limit: {}", limit);
    
    let bc = blockchain.read().await;
    info!("Current tenant blocks count: {}", bc.tenant_blocks.len());
    
    let blocks = bc.get_recent_tenant_blocks(limit);
    info!("Returning {} blocks", blocks.len());
    
    if blocks.is_empty() {
        warn!("No tenant blocks found! Check:");
        warn!("1. Tracker sending updates?");
        warn!("2. Browser clients creating transactions?");
        warn!("3. Network connectivity?");
    }
    
    Ok(warp::reply::json(&blocks))
}

async fn handle_tenants(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let tenants = bc.get_tenant_summaries();
    info!("Returning {} tenant summaries", tenants.len());
    
    Ok(warp::reply::json(&json!({
        "tenants": tenants,
        "total_tenants": tenants.len()
    })))
}

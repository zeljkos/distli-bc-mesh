use crate::blockchain::{Blockchain, TenantBlockchainUpdate};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use tracing::{info, warn};

pub async fn start_api_server(port: u16, blockchain: Arc<RwLock<Blockchain>>) {
    info!("Starting Enterprise API server on port {}", port);

    let blockchain_filter = warp::any().map(move || blockchain.clone());

    // Keep the tenant blockchain update endpoint for dashboard
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

    let delta_sync = warp::path("api")
        .and(warp::path("delta-sync"))
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(handle_delta_sync);

    let routes = tenant_blockchain_update
        .or(status)
        .or(blocks)
        .or(tenants)
        .or(delta_sync)
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
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received tenant blockchain update from network: {}", update.network_id);
    info!("Update contains {} blocks", update.new_blocks.len());
    
    let blocks_count = update.new_blocks.len();
    let mut transactions_count = 0;
    
    for block in &update.new_blocks {
        transactions_count += block.transactions.len();
    }
    
    {
        let mut bc = blockchain.write().await;
        bc.add_tenant_blocks(&update);
        info!("Stored tenant blocks for dashboard display");
    }
    
    info!("Processed {} blocks with {} transactions from {}", 
          blocks_count, transactions_count, update.network_id);
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Tenant blocks processed for enterprise tracking",
        "network_id": update.network_id,
        "blocks_processed": blocks_count,
        "transactions_processed": transactions_count
    })))
}

// Replace the handle_delta_sync function in src/enterprise_bc/api.rs
// Complete corrected handle_delta_sync function in src/enterprise_bc/api.rs

async fn handle_delta_sync(
    sync_payload: serde_json::Value,
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received delta sync from edge network");

    if let Some(sync_data) = sync_payload["sync_data"].as_object() {
        let network_id = sync_payload["network_id"].as_str().unwrap_or("unknown");
        let peer_id = sync_payload["peer_id"].as_str().unwrap_or("unknown");

        let empty_blocks = vec![];
        let empty_txs = vec![];

        let new_blocks = sync_data.get("new_blocks")
            .and_then(|b| b.as_array())
            .unwrap_or(&empty_blocks);

        let pending_txs = sync_data.get("pending_transactions")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty_txs);

        info!("Processing delta sync: {} new blocks, {} pending transactions from network {}",
              new_blocks.len(), pending_txs.len(), network_id);

        // Store the blocks in the correct order
        if !new_blocks.is_empty() {
            let mut parsed_blocks = Vec::new();
            
            // Parse and collect all blocks
            for block_value in new_blocks {
                if let Ok(block) = serde_json::from_value::<crate::blockchain::Block>(block_value.clone()) {
                    let block_height = block.height; // Get height before moving block
                    parsed_blocks.push(block);       // Move block here
                    info!("Parsed block #{} from network {} for processing", block_height, network_id);
                } else {
                    warn!("Failed to parse block from delta sync data");
                }
            }
            
            // Sort blocks by height (oldest first)
            parsed_blocks.sort_by(|a, b| a.height.cmp(&b.height));
            info!("Sorted {} blocks by height for sequential processing", parsed_blocks.len());
            
            // Get existing blocks for deduplication  
            let existing_blocks = {
                let bc = blockchain.read().await;
                bc.get_recent_tenant_blocks(1000)
            };
            
            let mut tenant_blocks = Vec::new();
            
            // Convert to TenantBlockData format with deduplication
            for block in parsed_blocks {
                let block_height = block.height; // Get height for logging
                
                // Check if this block already exists (by network_id + block_id)
                let already_exists = existing_blocks.iter().any(|existing| {
                    if let (Some(existing_network), Some(existing_block_id)) = 
                       (existing.get("network_id").and_then(|v| v.as_str()),
                        existing.get("block_id").and_then(|v| v.as_u64())) {
                        existing_network == network_id && 
                        existing_block_id == block.height as u64
                    } else {
                        false
                    }
                });
                
                if !already_exists {
                    let tenant_block = crate::blockchain::TenantBlockData {
                        block_id: block.height,
                        block_hash: block.hash,
                        transactions: block.transactions.iter().map(|tx| {
                            // Serialize complete transaction as JSON string
                            serde_json::to_string(tx).unwrap_or_else(|_| tx.id.clone())
                        }).collect(),
                        timestamp: block.timestamp,
                        previous_hash: block.previous_hash,
                        network_id: network_id.to_string(),
                    };
                    tenant_blocks.push(tenant_block);
                    info!("✅ New block #{} from network {} - will store", block_height, network_id);
                } else {
                    info!("⚠️ Duplicate block #{} from network {} - skipping", block_height, network_id);
                }
            }
            
            // Store the new blocks in correct order
            if !tenant_blocks.is_empty() {
                let update = crate::blockchain::TenantBlockchainUpdate {
                    network_id: network_id.to_string(),
                    peer_id: peer_id.to_string(),
                    new_blocks: tenant_blocks,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                
                {
                    let mut bc = blockchain.write().await;
                    bc.add_tenant_blocks(&update);
                    info!("✅ Stored {} NEW blocks from delta sync in correct order", update.new_blocks.len());
                }
            } else {
                info!("ℹ️ All blocks were duplicates - nothing new to store");
            }
        }

        Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": "Delta sync processed and stored in correct order",
            "network_id": network_id,
            "blocks_processed": new_blocks.len(),
            "blocks_stored": new_blocks.len(),
            "transactions_processed": pending_txs.len()
        })))
    } else {
        warn!("Invalid delta sync payload format");
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Invalid sync payload format"
        })))
    }
}
async fn handle_status(
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    
    let tenant_summaries = bc.get_tenant_summaries();
    let total_tenant_blocks = bc.get_recent_tenant_blocks(1000).len();
    
    // Use the new u32 methods  
    let pending_count = bc.get_pending_count();  // u32
    let validator_count = bc.get_validator_count();  // u32
    
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
    
    info!("Status request - tenant blocks: {}", total_tenant_blocks);
    Ok(warp::reply::json(&status))
}

async fn handle_blocks(
    query: BlocksQuery,
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let limit = query.limit.unwrap_or(20);
    info!("Blocks request with limit: {}", limit);
    
    let bc = blockchain.read().await;
    let blocks = bc.get_recent_tenant_blocks(limit);
    info!("Returning {} tenant blocks", blocks.len());
    
    if blocks.is_empty() {
        warn!("No tenant blocks found! Check:");
        warn!("1. Tracker sending updates?");
        warn!("2. Browser clients creating transactions?");
        warn!("3. Network connectivity?");
    }
    
    Ok(warp::reply::json(&blocks))
}

async fn handle_tenants(
    blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let tenants = bc.get_tenant_summaries();
    info!("Returning {} tenant summaries", tenants.len());
    
    Ok(warp::reply::json(&serde_json::json!({
        "tenants": tenants,
        "total_tenants": tenants.len()
    })))
}

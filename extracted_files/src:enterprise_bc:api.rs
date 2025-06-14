
use crate::enterprise_bc::EnterpriseBlockchain;
use crate::common::types::TenantUpdate;
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
    info!("Starting API server on port {}", port);

    let blockchain_filter = warp::any().map(move || blockchain.clone());

    // POST /api/tenant-update - Receive tenant updates
    let tenant_update = warp::path("api")
        .and(warp::path("tenant-update"))
        .and(warp::post())
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and_then(handle_tenant_update);

    // GET /api/status - Get blockchain status
    let status = warp::path("api")
        .and(warp::path("status"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_status);

    // GET /api/blocks - Get recent blocks
    let blocks = warp::path("api")
        .and(warp::path("blocks"))
        .and(warp::get())
        .and(warp::query::<BlocksQuery>())
        .and(blockchain_filter.clone())
        .and_then(handle_blocks);

    // GET /api/tenants - Get tenant summaries
    let tenants = warp::path("api")
        .and(warp::path("tenants"))
        .and(warp::get())
        .and(blockchain_filter.clone())
        .and_then(handle_tenants);

    // Health check
    let health = warp::path("health")
        .and(warp::get())
        .map(|| api_utils::health_check());

    let routes = tenant_update
        .or(status)
        .or(blocks)
        .or(tenants)
        .or(health)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

#[derive(serde::Deserialize)]
struct BlocksQuery {
    limit: Option<usize>,
}

async fn handle_tenant_update(
    update: TenantUpdate,
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Received tenant update for: {}", update.tenant_id);

    {
        let mut bc = blockchain.write().await;
        bc.add_tenant_update(update);
    }

    Ok(warp::reply::json(&json!({
        "status": "success",
        "message": "Tenant update received"
    })))
}

async fn handle_status(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let status = bc.get_blockchain_info();
    Ok(warp::reply::json(&status))
}

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

    let blocks = bc.chain[start_idx..].to_vec();
    Ok(warp::reply::json(&blocks))
}

async fn handle_tenants(
    blockchain: Arc<RwLock<EnterpriseBlockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    let bc = blockchain.read().await;
    let latest_block = bc.get_latest_block();

    let tenant_info: Vec<serde_json::Value> = latest_block.tenant_summaries.iter()
        .map(|summary| json!({
            "tenant_id": summary.tenant_id,
            "block_count": summary.block_count,
            "transaction_count": summary.transaction_count,
            "peer_count": summary.peer_count,
            "last_activity": summary.last_activity,
            "recent_messages": summary.messages
        }))
        .collect();

    Ok(warp::reply::json(&json!({
        "total_tenants": tenant_info.len(),
        "tenants": tenant_info
    })))
}

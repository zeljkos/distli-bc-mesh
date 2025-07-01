// Fixed tracker server - uses enterprise types throughout
use crate::common::types::{Message, NetworkPeer};
use crate::common::api_utils;
use crate::tracker::integration::EnterpriseIntegration;
use crate::enterprise_bc::{TenantBlockchainUpdate, TenantBlockData}; // Use enterprise type
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{WebSocket, Message as WsMessage};
use warp::Filter;
use serde_json::json;

pub type Networks = Arc<RwLock<HashMap<String, HashMap<String, NetworkPeer>>>>;
type GlobalPeers = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<WsMessage, warp::Error>>>>>;
pub type SharedIntegration = Arc<RwLock<Option<EnterpriseIntegration>>>;

pub struct Tracker {
    networks: Networks,
    global_peers: GlobalPeers,
    integration: SharedIntegration,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            networks: Arc::new(RwLock::new(HashMap::new())),
            global_peers: Arc::new(RwLock::new(HashMap::new())),
            integration: Arc::new(RwLock::new(None)),
        }
    }
    
    pub fn get_networks_ref(&self) -> Networks {
        self.networks.clone()
    }
    
    pub async fn set_integration(&self, integration: EnterpriseIntegration) {
        let mut int_lock = self.integration.write().await;
        *int_lock = Some(integration);
    }
    
    pub async fn run(&self) {
        let networks = self.networks.clone();
        let global_peers = self.global_peers.clone();
        let integration = self.integration.clone();
        
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map({
                let networks = networks.clone();
                let global_peers = global_peers.clone();
                let integration = integration.clone();
                move || (networks.clone(), global_peers.clone(), integration.clone())
            }))
            .map(|ws: warp::ws::Ws, (networks, global_peers, integration)| {
                ws.on_upgrade(move |socket| handle_peer(socket, networks, global_peers, integration))
            });

        // Use enterprise format throughout
        let blockchain_update_route = warp::path("api")
            .and(warp::path("blockchain-update"))
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map({
                let integration = integration.clone();
                move || integration.clone()
            }))
            .and_then(handle_blockchain_update);

        let networks_for_api = self.networks.clone();
        let api_route = warp::path("api")
            .and(warp::path("networks"))
            .and(warp::get())
            .and(warp::any().map(move || networks_for_api.clone()))
            .and_then(get_networks_info);

        let networks_for_list = self.networks.clone();
        let api_list_route = warp::path("api")
            .and(warp::path("network-list"))
            .and(warp::get())
            .and(warp::any().map(move || networks_for_list.clone()))
            .and_then(get_network_list);

        let health = warp::path("health")
            .and(warp::get())
            .map(|| api_utils::health_check());
            
        let static_files = warp::fs::dir("public");
        
        let routes = ws_route
            .or(blockchain_update_route)
            .or(api_route)
            .or(api_list_route)
            .or(health)
            .or(static_files)
            .with(warp::cors().allow_any_origin());
        
        println!("Multi-tenant tracker running on http://0.0.0.0:3030");
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    }
}

// Use enterprise format
async fn handle_blockchain_update(
    update: TenantBlockchainUpdate,
    integration: SharedIntegration
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received blockchain update from {}: {} new blocks", 
             update.network_id, update.new_blocks.len());
    
    let total_blocks = update.new_blocks.len();
    let total_transactions: usize = update.new_blocks.iter()
        .map(|block| block.transactions.len())
        .sum();
    
    // Log block details
    if !update.new_blocks.is_empty() {
        println!("Block details received:");
        for block in &update.new_blocks {
            if !block.transactions.is_empty() {
                println!("   Block #{}: {} transactions", block.block_id, block.transactions.len());
                for tx in &block.transactions {
                    println!("     - {}", tx);
                }
            }
        }
    }
    
    // Forward to enterprise BC
    {
        let mut int_lock = integration.write().await;
        if let Some(ref mut integration_instance) = int_lock.as_mut() {
            integration_instance.update_network_blockchain_state_with_update(&update).await;
            println!("Blockchain data forwarded to enterprise BC!");
        } else {
            println!("No enterprise integration configured");
        }
    }
    
    Ok(warp::reply::json(&json!({
        "status": "success",
        "message": "Blockchain update received and processed",
        "network_id": update.network_id,
        "new_blocks": total_blocks,
        "transactions": total_transactions,
        "immediate_processing": true
    })))
}

// Helper functions remain the same
async fn handle_peer(ws: WebSocket, networks: Networks, global_peers: GlobalPeers, integration: SharedIntegration) {
    let peer_id = Uuid::new_v4().to_string();
    let (mut peer_ws_tx, mut peer_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    
    let mut current_network: Option<String> = None;
    
    global_peers.write().await.insert(peer_id.clone(), tx.clone());
    let _ = send_network_list_update(&global_peers, &peer_id).await;

    let peer_id_clone = peer_id.clone();
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            if let Ok(msg) = message {
                if peer_ws_tx.send(msg).await.is_err() {
                    break;
                }
            }
        }
        println!("Peer {} message handler ended", &peer_id_clone[..8]);
    });
    
    while let Some(result) = peer_ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(text) = msg.to_str() {
                if let Ok(message) = serde_json::from_str::<Message>(text) {
                    match &message {
                        Message::JoinNetwork { network_id } => {
                            if let Some(old_network) = &current_network {
                                let mut networks_lock = networks.write().await;
                                if let Some(network_peers) = networks_lock.get_mut(old_network) {
                                    network_peers.remove(&peer_id);
                                    if network_peers.is_empty() {
                                        networks_lock.remove(old_network);
                                    }
                                }
                            }
                            
                            let network_peer = NetworkPeer {
                                peer_id: peer_id.clone(),
                                network_id: network_id.clone(),
                                sender: tx.clone(),
                                joined_at: std::time::Instant::now(),
                            };
                            
                            {
                                let mut networks_lock = networks.write().await;
                                networks_lock
                                    .entry(network_id.clone())
                                    .or_insert_with(HashMap::new)
                                    .insert(peer_id.clone(), network_peer);
                            }
                            
                            current_network = Some(network_id.clone());
                            
                            let peer_list = get_network_peers(&networks, network_id, &peer_id).await;
                            let _ = send_network_info(&networks, &peer_id, network_id).await;
                            let _ = send_to_peer_direct(&tx, Message::Peers { peers: peer_list }).await;
                            
                            broadcast_network_list_update(&networks, &global_peers).await;
                            
                            println!("Peer {} joined network: {}", &peer_id[..8], network_id);
                        }
                        Message::BlockchainUpdate { network_id, peer_id: update_peer_id, new_blocks, timestamp } => {
                            println!("Received blockchain update from {}: {} blocks", network_id, new_blocks.len());
                            
                            // Convert to TenantBlockchainUpdate format
                            let tenant_blocks: Vec<TenantBlockData> = new_blocks.iter()
                                .filter_map(|block| {
                                    Some(TenantBlockData {
                                        block_id: block["block_id"].as_u64()?,
                                        block_hash: block["block_hash"].as_str()?.to_string(),
                                        transactions: block["transactions"].as_array()?
                                            .iter()
                                            .filter_map(|tx| tx.as_str().map(|s| s.to_string()))
                                            .collect(),
                                        timestamp: block["timestamp"].as_u64()?,
                                        previous_hash: block["previous_hash"].as_str().unwrap_or("").to_string(),
                                    })
                                })
                                .collect();
                            
                            let update = TenantBlockchainUpdate {
                                network_id: network_id.clone(),
                                peer_id: update_peer_id.clone(),
                                new_blocks: tenant_blocks,
                                timestamp: *timestamp,
                            };
                            
                            // Forward to enterprise blockchain
                            let integration_clone = integration.clone();
                            tokio::spawn(async move {
                                let mut int_lock = integration_clone.write().await;
                                if let Some(ref mut integration_instance) = int_lock.as_mut() {
                                    integration_instance.update_network_blockchain_state_with_update(&update).await;
                                    println!("Blockchain data forwarded to enterprise BC from peer!");
                                } else {
                                    println!("No enterprise integration configured");
                                }
                            });
                        }
                        _ => {
                            if let Some(network_id) = &current_network {
                                handle_network_message(&networks, network_id, &peer_id, message).await;
                            }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    
    // Cleanup
    global_peers.write().await.remove(&peer_id);
    
    if let Some(network_id) = current_network {
        let mut networks_lock = networks.write().await;
        if let Some(network_peers) = networks_lock.get_mut(&network_id) {
            network_peers.remove(&peer_id);
            if network_peers.is_empty() {
                networks_lock.remove(&network_id);
                println!("Network {} removed (empty)", network_id);
            }
        }
        drop(networks_lock);
        
        broadcast_network_list_update(&networks, &global_peers).await;
    }
    
    println!("Peer {} disconnected", &peer_id[..8]);
}

async fn get_networks_info(networks: Networks) -> Result<impl warp::Reply, warp::Rejection> {
    let networks_lock = networks.read().await;
    let mut info = HashMap::new();
    
    for (network_id, peers) in networks_lock.iter() {
        info.insert(network_id.clone(), peers.len());
    }
    
    Ok(warp::reply::json(&info))
}

async fn get_network_list(networks: Networks) -> Result<impl warp::Reply, warp::Rejection> {
    let networks_lock = networks.read().await;
    let mut network_list = Vec::new();
    
    for (network_id, peers) in networks_lock.iter() {
        network_list.push(serde_json::json!({
            "id": network_id,
            "name": network_id,
            "peer_count": peers.len()
        }));
    }
    
    network_list.sort_by(|a, b| a["name"].as_str().unwrap().cmp(b["name"].as_str().unwrap()));
    
    Ok(warp::reply::json(&network_list))
}

async fn get_network_peers(networks: &Networks, network_id: &str, exclude_peer: &str) -> Vec<String> {
    let networks_lock = networks.read().await;
    if let Some(network_peers) = networks_lock.get(network_id) {
        network_peers
            .keys()
            .filter(|id| *id != exclude_peer)
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

async fn send_network_info(networks: &Networks, peer_id: &str, network_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let networks_lock = networks.read().await;
    if let Some(network_peers) = networks_lock.get(network_id) {
        if let Some(peer) = network_peers.get(peer_id) {
            let info_msg = Message::NetworkInfo {
                network_id: network_id.to_string(),
                peer_count: network_peers.len(),
            };
            let _ = send_to_peer_direct(&peer.sender, info_msg).await;
        }
    }
    Ok(())
}

async fn handle_network_message(networks: &Networks, network_id: &str, sender_id: &str, message: Message) {
    match message {
        Message::Offer { target, offer } => {
            let msg = Message::Offer { target: sender_id.to_string(), offer };
            let _ = send_to_network_peer(networks, network_id, &target, msg).await;
        }
        Message::Answer { target, answer } => {
            let msg = Message::Answer { target: sender_id.to_string(), answer };
            let _ = send_to_network_peer(networks, network_id, &target, msg).await;
        }
        Message::Candidate { target, candidate } => {
            let msg = Message::Candidate { target: sender_id.to_string(), candidate };
            let _ = send_to_network_peer(networks, network_id, &target, msg).await;
        }
        Message::Block { block } => {
            broadcast_to_network(networks, network_id, sender_id, Message::Block { block }).await;
        }
        Message::Transaction { transaction } => {
            broadcast_to_network(networks, network_id, sender_id, Message::Transaction { transaction }).await;
        }
        _ => {}
    }
}

async fn send_to_peer_direct(sender: &mpsc::UnboundedSender<Result<WsMessage, warp::Error>>, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(&message)?;
    sender.send(Ok(WsMessage::text(json)))?;
    Ok(())
}

async fn send_to_network_peer(networks: &Networks, network_id: &str, peer_id: &str, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    let networks_lock = networks.read().await;
    if let Some(network_peers) = networks_lock.get(network_id) {
        if let Some(peer) = network_peers.get(peer_id) {
            let json = serde_json::to_string(&message)?;
            peer.sender.send(Ok(WsMessage::text(json)))?;
        }
    }
    Ok(())
}

async fn broadcast_to_network(networks: &Networks, network_id: &str, sender_id: &str, message: Message) {
    let networks_lock = networks.read().await;
    if let Some(network_peers) = networks_lock.get(network_id) {
        let json = serde_json::to_string(&message).unwrap_or_default();
        
        for (peer_id, peer) in network_peers.iter() {
            if peer_id != sender_id {
                let _ = peer.sender.send(Ok(WsMessage::text(json.clone())));
            }
        }
    }
}

async fn send_network_list_update(global_peers: &GlobalPeers, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let networks: Vec<serde_json::Value> = Vec::new();
    
    let global_peers_lock = global_peers.read().await;
    if let Some(sender) = global_peers_lock.get(peer_id) {
        let update_msg = Message::NetworkListUpdate { networks };
        let json = serde_json::to_string(&update_msg)?;
        sender.send(Ok(WsMessage::text(json)))?;
    }
    Ok(())
}

async fn broadcast_network_list_update(networks: &Networks, global_peers: &GlobalPeers) {
    let networks_lock = networks.read().await;
    let mut network_list = Vec::new();
    
    for (network_id, peers) in networks_lock.iter() {
        network_list.push(serde_json::json!({
            "id": network_id,
            "name": network_id,
            "peer_count": peers.len()
        }));
    }
    
    network_list.sort_by(|a, b| a["name"].as_str().unwrap().cmp(b["name"].as_str().unwrap()));
    
    drop(networks_lock);
    
    let update_msg = Message::NetworkListUpdate { 
        networks: network_list 
    };
    
    let json = serde_json::to_string(&update_msg).unwrap_or_default();
    let global_peers_lock = global_peers.read().await;
    
    for sender in global_peers_lock.values() {
        let _ = sender.send(Ok(WsMessage::text(json.clone())));
    }
}

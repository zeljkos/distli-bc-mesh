// src/tracker/server.rs
use crate::blockchain::{Blockchain, Block, Transaction, TenantBlockchainUpdate, TenantBlockData};
use crate::tracker::integration::EnterpriseIntegration;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{WebSocket, Message as WsMessage};
use warp::Filter;
use std::collections::HashSet;

type Networks = Arc<RwLock<HashMap<String, HashMap<String, NetworkPeer>>>>;
type GlobalPeers = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<WsMessage, warp::Error>>>>>;

#[derive(Debug, Clone)]
pub struct NetworkPeer {
    pub peer_id: String,
    pub network_id: String,
    pub sender: mpsc::UnboundedSender<Result<WsMessage, warp::Error>>,
    pub joined_at: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "join_network")]
    JoinNetwork { network_id: String },
    
    #[serde(rename = "peers")]
    Peers { peers: Vec<String> },
    
    #[serde(rename = "offer")]
    Offer { target: String, offer: serde_json::Value },
    
    #[serde(rename = "answer")]
    Answer { target: String, answer: serde_json::Value },
    
    #[serde(rename = "candidate")]
    Candidate { target: String, candidate: serde_json::Value },
    
    #[serde(rename = "block")]
    Block { block: Block },
    
    #[serde(rename = "transaction")]
    Transaction { transaction: Transaction },
    
    #[serde(rename = "message")]
    ChatMessage { content: String, sender: String, timestamp: u64 },
    
    #[serde(rename = "network_info")]
    NetworkInfo { network_id: String, peer_count: usize },
    
    #[serde(rename = "network_list_update")]
    NetworkListUpdate { networks: Vec<serde_json::Value> },
    
    #[serde(rename = "blockchain_sync")]
    BlockchainSync { network_id: String, blocks: Vec<Block> },

    #[serde(rename = "enterprise_sync")]
    EnterpriseSync { network_id: String, sync_data: serde_json::Value },
}

pub struct Tracker {
    networks: Networks,
    global_peers: GlobalPeers,
    enterprise_blockchain: Arc<RwLock<Blockchain>>,
    enterprise_url: Option<String>,
    enterprise_integration: Option<Arc<RwLock<EnterpriseIntegration>>>,
    processed_blocks: Arc<RwLock<HashSet<String>>>, // Track processed block hashes
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            networks: Arc::new(RwLock::new(HashMap::new())),
            global_peers: Arc::new(RwLock::new(HashMap::new())),
            enterprise_blockchain: Arc::new(RwLock::new(Blockchain::new())),
            enterprise_url: None,
            enterprise_integration: None,
            processed_blocks: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    pub fn set_enterprise_url(&mut self, url: String) {
        self.enterprise_url = Some(url);
    }

    pub fn get_networks_ref(&self) -> Networks {
        self.networks.clone()
    }

    pub async fn set_integration(&mut self, integration: EnterpriseIntegration) {
        self.enterprise_integration = Some(Arc::new(RwLock::new(integration)));
    }
    
    pub async fn run(&self) {
        let networks = self.networks.clone();
        let global_peers = self.global_peers.clone();
        let enterprise_blockchain = self.enterprise_blockchain.clone();
        let enterprise_integration = self.enterprise_integration.clone();
        let processed_blocks = self.processed_blocks.clone(); // Pass to handler

        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map({
                let networks = networks.clone();
                let global_peers = global_peers.clone();
                let enterprise_blockchain = enterprise_blockchain.clone();
                let enterprise_integration = enterprise_integration.clone();
                let processed_blocks = processed_blocks.clone();
                move || (networks.clone(), global_peers.clone(), enterprise_blockchain.clone(), enterprise_integration.clone(), processed_blocks.clone())
            }))
            .map(|ws: warp::ws::Ws, (networks, global_peers, enterprise_blockchain, enterprise_integration, processed_blocks)| {
                ws.on_upgrade(move |socket| handle_peer(socket, networks, global_peers, enterprise_blockchain, enterprise_integration, processed_blocks))
            });

        let blockchain_sync_route = warp::path("api")
            .and(warp::path("blockchain-sync"))
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map({
                let enterprise_blockchain = enterprise_blockchain.clone();
                move || enterprise_blockchain.clone()
            }))
            .and_then(handle_blockchain_sync);

        let enterprise_update_route = warp::path("api")
            .and(warp::path("enterprise-update"))
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map({
                let networks = networks.clone();
                let global_peers = global_peers.clone();
                move || (networks.clone(), global_peers.clone())
            }))
            .and_then(handle_enterprise_update);

        // NEW: Cross-network trade endpoint
        let cross_network_trade_route = warp::path("api")
            .and(warp::path("cross-network-trade"))
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map({
                let networks = networks.clone();
                move || networks.clone()
            }))
            .and_then(handle_cross_network_trade);

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
            .map(|| warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            })));
            
        let static_files = warp::fs::dir("public");

        let order_book_broadcast_route = warp::path("api")
            .and(warp::path("order-book-broadcast"))
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map({
                let networks = networks.clone();
                move || networks.clone()
            }))
            .and_then(handle_order_book_broadcast);
        
        let routes = ws_route
            .or(blockchain_sync_route)
            .or(enterprise_update_route)
            .or(cross_network_trade_route)
            .or(order_book_broadcast_route)
            .or(api_route)
            .or(api_list_route)
            .or(health)
            .or(static_files)
            .with(warp::cors().allow_any_origin());

        println!("Tracker running on http://0.0.0.0:3030");
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    }
}

async fn handle_order_book_broadcast(
    order_update: serde_json::Value,
    networks: Networks
) -> Result<impl warp::Reply, warp::Rejection> {
    let message = Message::EnterpriseSync {
        network_id: "global".to_string(),
        sync_data: serde_json::json!({
            "type": "order_book_update",
            "orders": order_update["orders"]
        })
    };
    
    // Broadcast to all networks
    let networks_lock = networks.read().await;
    for (network_id, network_peers) in networks_lock.iter() {
        for (_, peer) in network_peers.iter() {
            let json = serde_json::to_string(&message).unwrap_or_default();
            let _ = peer.sender.send(Ok(WsMessage::text(json)));
        }
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Order book broadcast to all networks"
    })))
}



// NEW: Cross-network trade handler
async fn handle_cross_network_trade(
    trade_notification: serde_json::Value,
    networks: Networks
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received cross-network trade notification");
    
    let buyer_network = trade_notification["buyer_network"].as_str();
    let seller_network = trade_notification["seller_network"].as_str();
    
    if let (Some(buyer_net), Some(seller_net)) = (buyer_network, seller_network) {
        println!("Broadcasting trade execution to networks {} and {}", buyer_net, seller_net);
        
        // Create trade execution message
        let trade_execution = Message::EnterpriseSync {
            network_id: "cross_network".to_string(),
            sync_data: serde_json::json!({
                "type": "trade_execution",
                "trade": trade_notification
            })
        };
        
        // Broadcast to both networks
        broadcast_to_network(&networks, buyer_net, "enterprise", trade_execution.clone()).await;
        if buyer_net != seller_net {
            broadcast_to_network(&networks, seller_net, "enterprise", trade_execution).await;
        }
        
        println!("Trade execution broadcast to both networks");
        
        Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": "Cross-network trade broadcast to both networks",
            "buyer_network": buyer_net,
            "seller_network": seller_net
        })))
    } else {
        println!("Invalid cross-network trade notification - missing network IDs");
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Invalid trade notification format"
        })))
    }
}

async fn handle_enterprise_update(
    enterprise_update: serde_json::Value,
    (networks, _global_peers): (Networks, GlobalPeers)
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received enterprise update from validator");
    
    if let Some(network_id) = enterprise_update["network_id"].as_str() {
        println!("Broadcasting enterprise update to network: {}", network_id);
        
        let message = Message::BlockchainSync {
            network_id: network_id.to_string(),
            blocks: enterprise_update["blocks"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|b| serde_json::from_value(b.clone()).ok())
                .collect()
        };
        
        broadcast_to_network(&networks, network_id, "enterprise", message).await;
        
        println!("Enterprise update broadcast to {} network peers", network_id);
        
        Ok(warp::reply::json(&serde_json::json!({
            "status": "success",
            "message": "Enterprise update broadcast to network peers"
        })))
    } else {
        println!("Invalid enterprise update - missing network_id");
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": "Invalid enterprise update format"
        })))
    }
}

async fn handle_blockchain_sync(
    sync_message: serde_json::Value,
    _enterprise_blockchain: Arc<RwLock<Blockchain>>
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received blockchain sync from network");
    
    // Forward to enterprise validator if configured
    if let Some(url) = std::env::var("ENTERPRISE_BC_URL").ok() {
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let _ = client
                .post(&format!("{}/api/blockchain-sync", url))
                .json(&sync_message)
                .send()
                .await;
        });
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Blockchain synced"
    })))
}

async fn send_block_to_enterprise(block: &Block, network_id: &str, peer_id: &str) {
    if let Some(enterprise_url) = std::env::var("ENTERPRISE_BC_URL").ok() {
        println!("Converting P2P block #{} to enterprise format for network: {}", 
                 block.height, network_id);
        
        // Better transaction serialization with error handling
        let mut transactions = Vec::new();
        for tx in &block.transactions {
            match serde_json::to_string(tx) {
                Ok(tx_json) => {
                    transactions.push(tx_json);
                    println!("Serialized transaction: {} (type: {:?})", tx.id, tx.tx_type);
                }
                Err(e) => {
                    println!("Failed to serialize transaction {}: {}", tx.id, e);
                    // Fallback: create a minimal transaction representation
                    let fallback = serde_json::json!({
                        "id": tx.id,
                        "from": tx.from,
                        "to": tx.to,
                        "amount": tx.amount,
                        "timestamp": tx.timestamp,
                        "tx_type": format!("{:?}", tx.tx_type)
                    });
                    transactions.push(fallback.to_string());
                }
            }
        }
        
        // Create TenantBlockData with improved serialization
        let tenant_block = TenantBlockData {
            block_id: block.height,
            block_hash: block.hash.clone(),
            transactions,
            timestamp: block.timestamp,
            previous_hash: block.previous_hash.clone(),
            network_id: network_id.to_string(),
        };
        
        println!("TenantBlockData network_id: {}", tenant_block.network_id);
        
        // Create TenantBlockchainUpdate
        let update = TenantBlockchainUpdate {
            network_id: network_id.to_string(),
            peer_id: peer_id.to_string(),
            new_blocks: vec![tenant_block],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Send to enterprise validator
        let client = reqwest::Client::new();
        let url = format!("{}/api/tenant-blockchain-update", enterprise_url);
        
        match client.post(&url).json(&update).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Successfully sent block #{} from network {} to enterprise validator", 
                            block.height, network_id);
                } else {
                    println!("Failed to send to enterprise validator: HTTP {}", response.status());
                }
            }
            Err(e) => {
                println!("Error sending to enterprise validator: {}", e);
            }
        }
    } else {
        println!("No ENTERPRISE_BC_URL configured - skipping enterprise sync");
    }
}



async fn handle_peer(
    ws: WebSocket, 
    networks: Networks, 
    global_peers: GlobalPeers,
    _enterprise_blockchain: Arc<RwLock<Blockchain>>,
    enterprise_integration: Option<Arc<RwLock<EnterpriseIntegration>>>,
    processed_blocks: Arc<RwLock<HashSet<String>>>,
) {
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
                println!("Received WebSocket message: {}", text);
                if let Ok(message) = serde_json::from_str::<Message>(text) {
                    println!("Parsed message type: {:?}", std::mem::discriminant(&message));
                    match message.clone() {
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
                            
                            let peer_list = get_network_peers(&networks, &network_id, &peer_id).await;
                            let _ = send_network_info(&networks, &peer_id, &network_id).await;
                            let _ = send_to_peer_direct(&tx, Message::Peers { peers: peer_list }).await;
                            
                            broadcast_network_list_update(&networks, &global_peers).await;
                            
                            println!("Peer {} joined network: {}", &peer_id[..8], network_id);
                        }
                        Message::Block { block } => {
                            println!("Received Block message for block #{}", block.height);
                            if let Some(network_id) = &current_network {
                                // DEDUPLICATION CHECK - this is the key addition
                                let block_key = format!("{}:{}", network_id, block.hash);
                                
                                {
                                    let mut processed = processed_blocks.write().await;
                                    if processed.contains(&block_key) {
                                        println!("DUPLICATE BLOCK DETECTED: {} in network {} - SKIPPING", 
                                               block.hash, network_id);
                                        continue; // Skip processing this duplicate
                                    }
                                    processed.insert(block_key.clone());
                                    println!("NEW BLOCK: {} in network {} - PROCESSING", 
                                           block.hash, network_id);
                                }
                                
                                println!("Received block #{} from peer {} in network {}", 
                                        block.height, &peer_id[..8], network_id);
                                
                                // Broadcast to other peers in the network
                                broadcast_to_network(&networks, network_id, &peer_id, message.clone()).await;
                                
                                // Send to enterprise validator (only once now)
                                send_block_to_enterprise(&block, network_id, &peer_id).await;
                                
                                // Remove the duplicate enterprise integration processing
                                // Only keep one path to enterprise BC to avoid duplicates
                                
                                println!("Block #{} processed and forwarded to enterprise", block.height);
                            }
                        }
                        Message::EnterpriseSync { network_id, sync_data } => {
                            println!("Received enterprise sync request from network: {}", network_id);

                            if let Some(url) = std::env::var("ENTERPRISE_BC_URL").ok() {
                                let sync_payload = serde_json::json!({
                                    "type": "delta_sync",
                                    "network_id": network_id,
                                    "peer_id": peer_id,
                                    "sync_data": sync_data,
                                    "timestamp": std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs()
                                });

                                let url_clone = url.clone();
                                let payload_clone = sync_payload.clone();
                                tokio::spawn(async move {
                                    let client = reqwest::Client::new();
                                    match client
                                        .post(&format!("{}/api/delta-sync", url_clone))
                                        .json(&payload_clone)
                                        .send()
                                        .await
                                    {
                                        Ok(response) => {
                                            if response.status().is_success() {
                                                println!("Delta sync sent to enterprise blockchain");
                                            } else {
                                                println!("Enterprise sync failed: {}", response.status());
                                            }
                                        }
                                        Err(e) => {
                                            println!("Failed to send delta sync: {}", e);
                                        }
                                    }
                                });
                            } else {
                                println!("No enterprise blockchain URL configured");
                            }
                        }
                        Message::ChatMessage { content, sender, timestamp } => {
                            if let Some(network_id) = &current_network {
                                broadcast_to_network(&networks, network_id, &peer_id, message.clone()).await;
                                println!("Chat message '{}' from {} broadcast to network {}", content, sender, network_id);
                            }
                        }
                        Message::Transaction { transaction } => {
                            if let Some(network_id) = &current_network {
                                broadcast_to_network(&networks, network_id, &peer_id, message.clone()).await;
                                println!("Transaction {} broadcast to network {}", transaction.id, network_id);
                            }
                        }
                        _ => {
                            if let Some(network_id) = &current_network {
                                handle_network_message(&networks, network_id, &peer_id, message.clone()).await;
                            }
                        }
                    }
                } else {
                    println!("Failed to parse WebSocket message: {}", text);
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
///////
//////
////
////
////

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
        _ => {}
    }
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

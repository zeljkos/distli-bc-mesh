// Entry point for enterprise validator binary
use distli_mesh_bc::enterprise_bc::Validator;
use clap::Parser;
use std::env;

#[derive(Parser)]
#[command(name = "enterprise-validator")]
#[command(about = "Enterprise blockchain validator")]
struct Args {
    #[arg(short, long, default_value = "validator1")]
    id: String,
    
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    println!("Starting enterprise validator {} on port {}", args.id, args.port);
    
    let mut validator = Validator::new(
        args.id,
        args.port,
        get_peer_validators()
    ).await;
    
    // Set tracker URL for cross-network trade notifications
    if let Ok(tracker_url) = env::var("TRACKER_URL") {
        validator.set_tracker_url(tracker_url.clone()).await;
        println!("Tracker URL set to: {}", tracker_url);
    } else {
        // Default tracker URL
        let default_tracker = "http://192.168.200.133:3030".to_string();
        validator.set_tracker_url(default_tracker.clone()).await;
        println!("Using default tracker URL: {}", default_tracker);
    }
    
    validator.start().await;
}

fn get_peer_validators() -> Vec<String> {
    // In production, this would come from config
    // For Docker, we'll use service names
    let peers = env::var("VALIDATOR_PEERS")
        .unwrap_or_else(|_| "validator1:8080,validator2:8080,validator3:8080".to_string());
    
    peers.split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

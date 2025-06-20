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
    
    let validator = Validator::new(
        args.id,
        args.port,
        get_peer_validators()
    ).await;
    
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

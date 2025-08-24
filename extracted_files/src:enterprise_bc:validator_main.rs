use distli_mesh_bc::enterprise_bc::Validator;
use clap::Parser;

#[derive(Parser)]
#[command(name = "enterprise-validator")]
#[command(about = "Enterprise blockchain validator (Proof of Stake) cross-network order matching")]
struct Args {
    #[arg(short, long, default_value = "validator1")]
    id: String,
    
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    #[arg(short, long, default_value = "1000")]
    stake: u64,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    println!("Starting enterprise validator {} with cross-network order matching", args.id);
    println!("Port: {}, Stake: {}", args.port, args.stake);
    
    if let Some(tracker_url) = std::env::var("TRACKER_URL").ok() {
        println!("Tracker URL: {} (for cross-network trade broadcasting)", tracker_url);
    } else {
        println!("No TRACKER_URL set - cross-network trades won't be broadcast");
    }
    
    let validator = Validator::new(
        args.id,
        args.port,
        args.stake
    ).await;
    
    validator.start().await;
}

use distli_mesh_bc::enterprise_bc::Validator;
use clap::Parser;

#[derive(Parser)]
#[command(name = "enterprise-validator")]
#[command(about = "Enterprise blockchain validator with Proof of Stake")]
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
    
    println!("Starting enterprise validator {} on port {} with stake {}", 
             args.id, args.port, args.stake);
    
    let validator = Validator::new(
        args.id,
        args.port,
        args.stake
    ).await;
    
    validator.start().await;
}

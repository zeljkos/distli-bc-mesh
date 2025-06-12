// Entry point for enterprise dashboard binary
use distli_mesh_bc::enterprise_bc::dashboard;
use clap::Parser;

#[derive(Parser)]
#[command(name = "enterprise-dashboard")]
#[command(about = "Enterprise blockchain dashboard")]
struct Args {
    #[arg(short, long, default_value = "9090")]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    println!("Starting enterprise dashboard on port {}", args.port);
    
    dashboard::start_dashboard(args.port).await;
}

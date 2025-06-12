// Entry point for tracker binary
use distli_mesh_bc::tracker::{Tracker, EnterpriseIntegration};
use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    println!("Starting distli-mesh-bc tracker with enterprise integration...");
    
    let tracker = Tracker::new();
    let networks = tracker.get_networks_ref();
    
    // Start enterprise integration if URL is provided
    let enterprise_url = env::var("ENTERPRISE_BC_URL")
        .unwrap_or_else(|_| "http://192.168.200.133:8080".to_string());
    
    if !enterprise_url.is_empty() {
        let mut enterprise = EnterpriseIntegration::new(enterprise_url);
        let networks_clone = networks.clone();
        
        tokio::spawn(async move {
            enterprise.start_reporting_loop(networks_clone).await;
        });
        
        println!("Enterprise blockchain integration enabled");
    }
    
    tracker.run().await;
}

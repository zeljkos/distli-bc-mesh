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
        // Set up integration for blockchain updates (for the API handler)
        let enterprise_for_server = EnterpriseIntegration::new(enterprise_url.clone());
        tracker.set_integration(enterprise_for_server).await;
        
        // Start reporting loop with separate integration instance
        let mut enterprise_reporter = EnterpriseIntegration::new(enterprise_url);
        let networks_clone = networks.clone();
        
        tokio::spawn(async move {
            enterprise_reporter.start_reporting_loop(networks_clone).await;
        });
        
        println!("Enterprise blockchain integration enabled");
    } else {
        println!("No enterprise blockchain URL provided - running in standalone mode");
    }
    
    // Start the tracker server
    tracker.run().await;
}

use distli_mesh_bc::tracker::{Tracker, EnterpriseIntegration};
use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    println!("Starting distli-mesh-bc tracker with enterprise integration...");
    
    let mut tracker = Tracker::new();
    
    // Start enterprise integration if URL is provided
    let enterprise_url = env::var("ENTERPRISE_BC_URL")
        .unwrap_or_else(|_| "http://192.168.200.133:8080".to_string());
    
    if !enterprise_url.is_empty() {
        println!("Enterprise blockchain integration enabled: {}", enterprise_url);
        
        // Set up integration
        let enterprise_integration = EnterpriseIntegration::new(enterprise_url.clone());
        tracker.set_integration(enterprise_integration).await;
        
        // Start reporting loop in background
        let networks = tracker.get_networks_ref();
        let enterprise_url_clone = enterprise_url.clone();
        
        tokio::spawn(async move {
            let mut reporter = EnterpriseIntegration::new(enterprise_url_clone);
            reporter.start_reporting_loop(networks).await;
        });
        
        tracker.set_enterprise_url(enterprise_url);
    } else {
        println!("No enterprise blockchain URL provided - running in standalone mode");
    }
    
    // Start the tracker server
    tracker.run().await;
}

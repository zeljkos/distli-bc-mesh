mod blockchain;
mod tracker;
mod types;

use tracker::Tracker;

#[tokio::main]
async fn main() {
    println!("Starting distli-mesh-bc...");
    
    let tracker = Tracker::new();
    tracker.run().await;
}

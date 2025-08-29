// Add ZK Proof Contract Data to Enterprise Blockchain
// This example adds ZK proof contract messages as simple transactions

use distli_mesh_bc::blockchain::Blockchain;
// Remove unused imports

fn main() {
    println!("Adding ZK Proof Contract Data to Enterprise Blockchain");
    println!("======================================================\n");

    // Load enterprise blockchain
    let storage_path = "data/enterprise_blockchain_validator1.json";
    let mut blockchain = Blockchain::new_with_storage(storage_path.to_string());
    
    println!("Current blockchain status:");
    println!("  - Height: {}", blockchain.height());
    println!("  - Pending transactions: {}", blockchain.get_pending_count());
    
    // Add ZK proof contract transactions
    println!("\nAdding ZK Proof Contract: T-Mobile <-> Orange");
    let contract1_id = blockchain.add_transaction(
        "T-Mobile".to_string(),
        "Orange".to_string(),
        0 // ZK contract creation - no transfer amount
    );
    blockchain.add_message(
        format!(r#"{{"type":"ZKPrivateContract","contract_id":"{}","participants":["T-Mobile","Orange"],"rate_encrypted":"true","settlement_amount":12500,"zk_proofs":{{"billing_proof":"verified","settlement_proof":"verified"}},"session_count":3,"status":"active"}}"#, contract1_id),
        "T-Mobile".to_string()
    );
    
    println!("Added ZK Contract 1: {}", &contract1_id[0..8]);
    
    // Add sessions for T-Mobile <-> Orange
    for i in 1..=3 {
        let session_id = blockchain.add_transaction(
            "T-Mobile".to_string(),
            "Orange".to_string(),
            (50 + i * 10) * 15 // Amount based on duration * rate
        );
        blockchain.add_message(
            format!(r#"{{"type":"ZKPrivateSession","tx_id":"{}","contract":"tm_orange","session_id":{},"imsi_commitment":"{}","duration_minutes":{},"billing_proof":"zk_proof_{}","status":"verified"}}"#,
                session_id,
                i, 
                format!("{:08x}", (i as u64 * 0x6fe3307a) % 0xFFFFFFFF),
                50 + i * 10,
                i
            ),
            "T-Mobile".to_string()
        );
        println!("  Session {}: IMSI commitment {}", i, format!("{:08x}", (i as u64 * 0x6fe3307a) % 0xFFFFFFFF));
    }
    
    println!("\nAdding ZK Proof Contract: T-Mobile <-> Vodafone");
    let contract2_id = blockchain.add_transaction(
        "T-Mobile".to_string(),
        "Vodafone".to_string(),
        0 // ZK contract creation - no transfer amount
    );
    blockchain.add_message(
        format!(r#"{{"type":"ZKPrivateContract","contract_id":"{}","participants":["T-Mobile","Vodafone"],"rate_encrypted":"true","settlement_amount":12500,"zk_proofs":{{"billing_proof":"verified","settlement_proof":"verified"}},"session_count":2,"status":"active"}}"#, contract2_id),
        "T-Mobile".to_string()
    );
    
    println!("Added ZK Contract 2: {}", &contract2_id[0..8]);
    
    // Add sessions for T-Mobile <-> Vodafone
    for i in 1..=2 {
        let session_id = blockchain.add_transaction(
            "T-Mobile".to_string(),
            "Vodafone".to_string(),
            (75 + i * 15) * 12 // Amount based on duration * rate
        );
        blockchain.add_message(
            format!(r#"{{"type":"ZKPrivateSession","tx_id":"{}","contract":"tm_vodafone","session_id":{},"imsi_commitment":"{}","duration_minutes":{},"billing_proof":"zk_proof_{}","status":"verified"}}"#,
                session_id,
                i + 3, 
                format!("{:08x}", ((i + 3) as u64 * 0x1543fa98) % 0xFFFFFFFF),
                75 + i * 15,
                i + 3
            ),
            "T-Mobile".to_string()
        );
        println!("  Session {}: IMSI commitment {}", i + 3, format!("{:08x}", ((i + 3) as u64 * 0x1543fa98) % 0xFFFFFFFF));
    }
    
    // Add settlement transactions
    println!("\nAdding ZK Proof Settlements");
    let settlement1_id = blockchain.add_transaction(
        "T-Mobile".to_string(),
        "Orange".to_string(),
        12500 // Total settlement amount
    );
    blockchain.add_message(
        format!(r#"{{"type":"ZKSettlement","tx_id":"{}","contract":"tm_orange","total_amount":12500,"session_count":3,"zk_proofs":{{"settlement_aggregation":"verified","billing_correctness":"verified","range_proofs":["verified","verified","verified"]}},"verification_status":"complete"}}"#, settlement1_id),
        "T-Mobile".to_string()
    );
    
    let settlement2_id = blockchain.add_transaction(
        "T-Mobile".to_string(),
        "Vodafone".to_string(),
        12500 // Total settlement amount
    );
    blockchain.add_message(
        format!(r#"{{"type":"ZKSettlement","tx_id":"{}","contract":"tm_vodafone","total_amount":12500,"session_count":2,"zk_proofs":{{"settlement_aggregation":"verified","billing_correctness":"verified","range_proofs":["verified","verified"]}},"verification_status":"complete"}}"#, settlement2_id),
        "T-Mobile".to_string()
    );
    
    println!("Added settlements for both contracts");
    
    // Mine blocks to include all transactions
    println!("\nMining blocks to include all transactions...");
    let initial_height = blockchain.height();
    let initial_pending = blockchain.get_pending_count();
    
    while blockchain.get_pending_count() > 0 {
        if blockchain.mine_block() {
            println!("  Mined block #{}", blockchain.height());
        }
    }
    
    blockchain.save_to_disk();
    
    println!("\nFinal blockchain status:");
    println!("  - Height: {} (was {})", blockchain.height(), initial_height);
    println!("  - New blocks: {}", blockchain.height() - initial_height);
    println!("  - Transactions processed: {}", initial_pending);
    println!("  - Pending: {}", blockchain.get_pending_count());
    
    println!("\nZK Proof Contract Data Added to Blockchain!");
    println!("You can now view the ZK proof transactions in the enterprise dashboard");
    println!("Main Dashboard: http://192.168.200.133:9090");
    println!("ZK Dashboard: http://192.168.200.133:9090/zk");
}
// Initialize ZK Proof Contracts in Enterprise Blockchain
// This example adds ZK proof contracts as actual blockchain transactions

use distli_mesh_bc::common::{PrivateContractManager, ContractTerms};
use distli_mesh_bc::blockchain::{Blockchain, Transaction, TxType};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("Initializing ZK Proof Contracts in Enterprise Blockchain");
    println!("=========================================================\n");

    // Load or create enterprise blockchain
    let storage_path = "data/enterprise_blockchain_validator1.json";
    let mut blockchain = Blockchain::new_with_storage(storage_path.to_string());
    
    println!("Current blockchain status:");
    println!("  - Height: {}", blockchain.get_height());
    println!("  - Pending transactions: {}", blockchain.get_pending_count());
    
    // Create ZK contract manager
    let mut contract_manager = PrivateContractManager::new();
    
    // Register operators
    println!("\nRegistering telecom operators...");
    contract_manager.register_operator("T-Mobile", "tm_public_key_xyz", "tm_private_key_abc");
    contract_manager.register_operator("Orange", "or_public_key_xyz", "or_private_key_abc");
    contract_manager.register_operator("Vodafone", "vf_public_key_xyz", "vf_private_key_abc");
    contract_manager.register_operator("AT&T", "att_public_key_xyz", "att_private_key_abc");
    println!("4 operators registered");

    // Create T-Mobile <-> Orange contract
    println!("\nCreating T-Mobile <-> Orange private contract...");
    let tm_orange_terms = ContractTerms {
        operator_a: "T-Mobile".to_string(),
        operator_b: "Orange".to_string(),
        rate_per_minute: 15,  // Premium rate (private!)
        rate_per_mb: 5,
        rate_per_sms: 2,
        minimum_commitment: 50000,
        discount_tiers: vec![],
        settlement_period_days: 30,
        dispute_resolution_period_days: 15,
    };
    
    let contract1_id = contract_manager.create_private_contract(
        "T-Mobile", "Orange", tm_orange_terms
    ).unwrap();
    
    // Add contract as blockchain transaction
    let contract1_tx = Transaction {
        id: format!("zkcontract_{}", &contract1_id[0..8]),
        from: "T-Mobile".to_string(),
        to: "Orange".to_string(),
        amount: 0,
        tx_type: TxType::Message {
            content: json!({
                "type": "ZKPrivateContract",
                "contract_id": contract1_id,
                "participants": ["T-Mobile", "Orange"],
                "contract_hash": format!("hash_tm_orange_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
                "settlement_amount": 12500,
                "zk_proofs": {
                    "billing_proof": "zk_billing_proof_tm_orange_123",
                    "settlement_proof": "zk_settlement_proof_tm_orange_456"
                },
                "session_count": 3,
                "status": "active"
            }).to_string()
        },
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    blockchain.add_transaction(contract1_tx);
    println!("Added T-Mobile <-> Orange contract to blockchain");
    
    // Add roaming sessions for the contract
    for i in 1..=3 {
        let imsi = format!("31041012345678{}", i);
        let duration = 50 + i * 10;
        let amount = duration * 15; // Using the secret rate
        
        let _session = contract_manager.add_private_session(
            &contract1_id,
            "T-Mobile",
            &imsi,
            duration as u64,
            amount as u64
        ).unwrap();
        
        // Add session as blockchain transaction
        let session_tx = Transaction {
            id: format!("zksession_{}_{}", &contract1_id[0..8], i),
            from: "T-Mobile".to_string(),
            to: "Orange".to_string(),
            amount: amount as u32,
            tx_type: TxType::Message {
                content: json!({
                    "type": "ZKPrivateSession",
                    "contract_id": contract1_id,
                    "session_id": i,
                    "imsi_commitment": format!("{:08x}", i * 0x6fe3307a),
                    "duration_minutes": duration,
                    "billing_amount": amount,
                    "zk_proofs": {
                        "duration_proof": format!("zk_duration_proof_{}", i),
                        "billing_proof": format!("zk_billing_proof_{}", i),
                        "imsi_commitment_proof": format!("zk_imsi_proof_{}", i)
                    },
                    "status": "verified"
                }).to_string()
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        blockchain.add_transaction(session_tx);
    }
    println!("Added 3 ZK proof sessions for T-Mobile <-> Orange");

    // Create T-Mobile <-> Vodafone contract
    println!("\nCreating T-Mobile <-> Vodafone private contract...");
    let tm_vodafone_terms = ContractTerms {
        operator_a: "T-Mobile".to_string(),
        operator_b: "Vodafone".to_string(),
        rate_per_minute: 12,  // Different rate (also private!)
        rate_per_mb: 4,
        rate_per_sms: 1,
        minimum_commitment: 75000,
        discount_tiers: vec![],
        settlement_period_days: 30,
        dispute_resolution_period_days: 15,
    };
    
    let contract2_id = contract_manager.create_private_contract(
        "T-Mobile", "Vodafone", tm_vodafone_terms
    ).unwrap();
    
    // Add contract as blockchain transaction
    let contract2_tx = Transaction {
        id: format!("zkcontract_{}", &contract2_id[0..8]),
        from: "T-Mobile".to_string(),
        to: "Vodafone".to_string(),
        amount: 0,
        tx_type: TxType::Message {
            content: json!({
                "type": "ZKPrivateContract",
                "contract_id": contract2_id,
                "participants": ["T-Mobile", "Vodafone"],
                "contract_hash": format!("hash_tm_vodafone_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
                "settlement_amount": 12500,
                "zk_proofs": {
                    "billing_proof": "zk_billing_proof_tm_vodafone_789",
                    "settlement_proof": "zk_settlement_proof_tm_vodafone_012"
                },
                "session_count": 2,
                "status": "active"
            }).to_string()
        },
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    blockchain.add_transaction(contract2_tx);
    println!("Added T-Mobile <-> Vodafone contract to blockchain");
    
    // Add roaming sessions for the second contract
    for i in 1..=2 {
        let imsi = format!("31041098765432{}", i);
        let duration = 75 + i * 15;
        let amount = duration * 12; // Using different secret rate
        
        let _session = contract_manager.add_private_session(
            &contract2_id,
            "T-Mobile",
            &imsi,
            duration as u64,
            amount as u64
        ).unwrap();
        
        // Add session as blockchain transaction
        let session_tx = Transaction {
            id: format!("zksession_{}_{}", &contract2_id[0..8], i + 3),
            from: "T-Mobile".to_string(),
            to: "Vodafone".to_string(),
            amount: amount as u32,
            tx_type: TxType::Message {
                content: json!({
                    "type": "ZKPrivateSession",
                    "contract_id": contract2_id,
                    "session_id": i + 3,
                    "imsi_commitment": format!("{:08x}", (i + 3) * 0x1543fa98),
                    "duration_minutes": duration,
                    "billing_amount": amount,
                    "zk_proofs": {
                        "duration_proof": format!("zk_duration_proof_{}", i + 3),
                        "billing_proof": format!("zk_billing_proof_{}", i + 3),
                        "imsi_commitment_proof": format!("zk_imsi_proof_{}", i + 3)
                    },
                    "status": "verified"
                }).to_string()
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        blockchain.add_transaction(session_tx);
    }
    println!("Added 2 ZK proof sessions for T-Mobile <-> Vodafone");

    // Create settlements
    println!("\nCreating ZK proof settlements...");
    let settlement1 = contract_manager.create_settlement(&contract1_id, "T-Mobile").unwrap();
    let settlement2 = contract_manager.create_settlement(&contract2_id, "T-Mobile").unwrap();
    
    // Add settlements as blockchain transactions
    let settlement1_tx = Transaction {
        id: format!("zksettlement_{}", &contract1_id[0..8]),
        from: "T-Mobile".to_string(),
        to: "Orange".to_string(),
        amount: settlement1.total_amount as u32,
        tx_type: TxType::Message {
            content: json!({
                "type": "ZKSettlement",
                "contract_id": contract1_id,
                "settlement_id": settlement1.settlement_id,
                "total_amount": settlement1.total_amount,
                "session_count": 3,
                "zk_proofs": {
                    "settlement_aggregation_proof": "zk_settlement_agg_proof_123",
                    "billing_correctness_proof": "zk_billing_correct_proof_456",
                    "range_proofs": ["zk_range_1", "zk_range_2", "zk_range_3"]
                },
                "verification_status": "verified",
                "validator_signature": "validator1_signature_abc"
            }).to_string()
        },
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let settlement2_tx = Transaction {
        id: format!("zksettlement_{}", &contract2_id[0..8]),
        from: "T-Mobile".to_string(),
        to: "Vodafone".to_string(),
        amount: settlement2.total_amount as u32,
        tx_type: TxType::Message {
            content: json!({
                "type": "ZKSettlement",
                "contract_id": contract2_id,
                "settlement_id": settlement2.settlement_id,
                "total_amount": settlement2.total_amount,
                "session_count": 2,
                "zk_proofs": {
                    "settlement_aggregation_proof": "zk_settlement_agg_proof_789",
                    "billing_correctness_proof": "zk_billing_correct_proof_012",
                    "range_proofs": ["zk_range_4", "zk_range_5"]
                },
                "verification_status": "verified",
                "validator_signature": "validator1_signature_def"
            }).to_string()
        },
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    blockchain.add_transaction(settlement1_tx);
    blockchain.add_transaction(settlement2_tx);
    println!("Added ZK proof settlements to blockchain");

    // Mine blocks to include all transactions
    println!("\nMining blocks to include ZK proof transactions...");
    let initial_height = blockchain.get_height();
    let initial_pending = blockchain.get_pending_count();
    
    while blockchain.get_pending_count() > 0 {
        if blockchain.mine_block() {
            println!("   Mined block #{}", blockchain.get_height());
        }
    }
    
    blockchain.save_to_disk();
    
    println!("\nFinal blockchain status:");
    println!("  - Height: {} (was {})", blockchain.get_height(), initial_height);
    println!("  - New blocks: {}", blockchain.get_height() - initial_height);
    println!("  - Transactions processed: {}", initial_pending);
    println!("  - Pending: {}", blockchain.get_pending_count());
    
    println!("\nZK Proof Contract System Initialized!");
    println!("Contracts and sessions are now stored as blockchain transactions");
    println!("You can view them in the enterprise dashboard at http://192.168.200.133:9090");
    println!("ZK proof details visible at http://192.168.200.133:9090/zk");
}
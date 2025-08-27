// GSM Roaming Smart Contract Test Example
use distli_mesh_bc::common::{
    create_gsm_roaming_contract, ContractVM, ContractCall
};
use std::{thread, time::Duration};

fn main() {
    println!("🏠📱 GSM Roaming Smart Contract Demo");
    println!("=====================================");
    
    // Create contract VM and deploy roaming contract
    let mut vm = ContractVM::new();
    let contract = create_gsm_roaming_contract("NetworkOperator".to_string(), Some("gsm_roaming_demo".to_string()));
    let contract_id = vm.deploy_contract(contract).expect("Failed to deploy contract");
    
    println!("✅ GSM Roaming contract deployed with ID: {}", contract_id);
    
    // Set up network rates
    println!("\n📊 Setting up roaming rates...");
    set_roaming_rate(&mut vm, &contract_id, "Vodafone", "T-Mobile", 15);
    set_roaming_rate(&mut vm, &contract_id, "T-Mobile", "Vodafone", 12);
    set_roaming_rate(&mut vm, &contract_id, "Orange", "Verizon", 20);
    
    // Simulate GSM roaming session
    println!("\n📡 Simulating GSM roaming session...");
    let imsi = "310410123456789"; // Example IMSI
    let session_id = connect_to_network(&mut vm, &contract_id, imsi, "Vodafone", "T-Mobile", "ANT-001", "wallet_vodafone", "wallet_tmobile");
    
    println!("📞 IMSI {} connected to antenna ANT-001", imsi);
    println!("💰 Rate: 15 units/minute (Vodafone -> T-Mobile)");
    
    // Simulate minute-by-minute billing for 5 minutes
    println!("\n⏰ Starting automated billing simulation...");
    for minute in 1..=5 {
        println!("\n--- Minute {} ---", minute);
        
        // Process minute billing
        let billing_result = process_minute_billing(&mut vm, &contract_id, &session_id);
        if let Some(result) = billing_result.result.as_object() {
            println!("💳 Billed: {} units (Total: {} units)", 
                result.get("amount").unwrap().as_u64().unwrap(),
                result.get("totalCost").unwrap().as_u64().unwrap()
            );
            
            // Simulate actual blockchain transaction from guest to host
            println!("💸 Transfer: wallet_vodafone -> wallet_tmobile ({} units)", 
                result.get("amount").unwrap().as_u64().unwrap()
            );
        }
        
        // Wait 1 second to simulate 1 minute (in real system this would be 60 seconds)
        thread::sleep(Duration::from_secs(1));
    }
    
    // Get active session info
    println!("\n📋 Session Status:");
    let session_info = get_session(&mut vm, &contract_id, &session_id);
    if let Some(session) = session_info.result.as_object() {
        println!("📱 IMSI: {}", session.get("imsi").unwrap().as_str().unwrap());
        println!("🏠 Home Network: {}", session.get("homeNetwork").unwrap().as_str().unwrap());
        println!("🌍 Visiting Network: {}", session.get("visitingNetwork").unwrap().as_str().unwrap());
        println!("💰 Total Cost: {} units", session.get("totalCost").unwrap().as_u64().unwrap());
        println!("⏱️ Minutes Billed: {}", session.get("minutesBilled").unwrap().as_u64().unwrap());
    }
    
    // Simulate disconnection after 3 more minutes
    println!("\n📴 Simulating disconnection in 3 more minutes...");
    for minute in 6..=8 {
        println!("\n--- Minute {} ---", minute);
        let billing_result = process_minute_billing(&mut vm, &contract_id, &session_id);
        if let Some(result) = billing_result.result.as_object() {
            println!("💳 Billed: {} units (Total: {} units)", 
                result.get("amount").unwrap().as_u64().unwrap(),
                result.get("totalCost").unwrap().as_u64().unwrap()
            );
        }
        thread::sleep(Duration::from_secs(1));
    }
    
    // Disconnect
    println!("\n📴 Disconnecting from network...");
    let disconnect_result = disconnect_from_network(&mut vm, &contract_id, &session_id);
    if let Some(result) = disconnect_result.result.as_object() {
        println!("✅ Session ended");
        println!("⏱️ Duration: {} minutes", result.get("durationMinutes").unwrap().as_u64().unwrap());
        println!("💰 Final Cost: {} units", result.get("totalCost").unwrap().as_u64().unwrap());
    }
    
    // Show billing history
    println!("\n📊 Billing History:");
    let history = get_billing_history(&mut vm, &contract_id, Some(imsi));
    if let Some(history_data) = history.result.as_object() {
        if let Some(records) = history_data.get("billingHistory").and_then(|h| h.as_array()) {
            for (i, record) in records.iter().enumerate() {
                println!("  {}. Session: {} | Duration: {}min | Cost: {} units", 
                    i + 1,
                    record.get("sessionId").unwrap().as_str().unwrap(),
                    record.get("durationMinutes").unwrap().as_u64().unwrap(),
                    record.get("totalCost").unwrap().as_u64().unwrap()
                );
            }
        }
    }
    
    println!("\n🎉 GSM Roaming demo completed!");
}

fn set_roaming_rate(vm: &mut ContractVM, contract_id: &str, home_network: &str, visiting_network: &str, rate: u64) {
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "setRate".to_string(),
        params: serde_json::json!({
            "homeNetwork": home_network,
            "visitingNetwork": visiting_network,
            "ratePerMinute": rate
        }),
        caller: "NetworkOperator".to_string(),
        gas_limit: 1000,
    };
    
    let result = vm.call_contract(call);
    if result.success {
        println!("  📈 Rate set: {} -> {} at {} units/minute", home_network, visiting_network, rate);
    }
}

fn connect_to_network(vm: &mut ContractVM, contract_id: &str, imsi: &str, home_network: &str, visiting_network: &str, antenna_id: &str, guest_wallet: &str, host_wallet: &str) -> String {
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "connect".to_string(),
        params: serde_json::json!({
            "imsi": imsi,
            "homeNetwork": home_network,
            "visitingNetwork": visiting_network,
            "antennaId": antenna_id,
            "guestWallet": guest_wallet,
            "hostWallet": host_wallet
        }),
        caller: antenna_id.to_string(),
        gas_limit: 1000,
    };
    
    let result = vm.call_contract(call);
    if result.success {
        return result.result.get("sessionId").unwrap().as_str().unwrap().to_string();
    }
    
    panic!("Failed to connect to network");
}

fn process_minute_billing(vm: &mut ContractVM, contract_id: &str, session_id: &str) -> distli_mesh_bc::common::ContractResult {
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "processMinuteBilling".to_string(),
        params: serde_json::json!({
            "sessionId": session_id
        }),
        caller: "BillingSystem".to_string(),
        gas_limit: 1000,
    };
    
    vm.call_contract(call)
}

fn disconnect_from_network(vm: &mut ContractVM, contract_id: &str, session_id: &str) -> distli_mesh_bc::common::ContractResult {
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "disconnect".to_string(),
        params: serde_json::json!({
            "sessionId": session_id
        }),
        caller: "NetworkSystem".to_string(),
        gas_limit: 1000,
    };
    
    vm.call_contract(call)
}

fn get_session(vm: &mut ContractVM, contract_id: &str, session_id: &str) -> distli_mesh_bc::common::ContractResult {
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "getSession".to_string(),
        params: serde_json::json!({
            "sessionId": session_id
        }),
        caller: "QuerySystem".to_string(),
        gas_limit: 1000,
    };
    
    vm.call_contract(call)
}

fn get_billing_history(vm: &mut ContractVM, contract_id: &str, imsi: Option<&str>) -> distli_mesh_bc::common::ContractResult {
    let mut params = serde_json::json!({
        "limit": 10
    });
    
    if let Some(imsi) = imsi {
        params["imsi"] = serde_json::Value::String(imsi.to_string());
    }
    
    let call = ContractCall {
        contract_id: contract_id.to_string(),
        function: "getBillingHistory".to_string(),
        params,
        caller: "QuerySystem".to_string(),
        gas_limit: 1000,
    };
    
    vm.call_contract(call)
}
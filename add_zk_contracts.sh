#!/bin/bash

set -xv

# ZK Proof Contracts Management Script
# This script demonstrates adding ZK proof contracts via API and checking results

API_BASE="http://192.168.200.133:8080"
DASHBOARD_BASE="http://192.168.200.133:9090"

echo "=========================================="
echo "ZK Proof Contracts Management Script"
echo "=========================================="
echo ""

# Function to check API connectivity
check_api() {
    echo "Checking API connectivity..."
    response=$(curl -s -w "%{http_code}" "$API_BASE/health" -o /dev/null)
    if [ "$response" -eq 200 ]; then
        echo "✓ API is accessible at $API_BASE"
    else
        echo "✗ API not accessible at $API_BASE (HTTP $response)"
        exit 1
    fi
    echo ""
}

# Function to get current blockchain status
check_blockchain_status() {
    echo "Checking current blockchain status..."
    
    # Get current blocks
    echo "Current blocks in blockchain:"
    curl -s "$API_BASE/api/blocks?limit=10" | jq -r '.[] | "Block #\(.block_id): \(.network_id) - \(.block_hash[0:16])... (\(.transactions | length) transactions)"' 2>/dev/null || {
        echo "Raw response:"
        curl -s "$API_BASE/api/blocks?limit=10"
    }
    
    # Get status
    echo ""
    echo "Blockchain status:"
    curl -s "$API_BASE/api/status" | jq . 2>/dev/null || {
        echo "Raw status:"
        curl -s "$API_BASE/api/status"
    }
    echo ""
    echo "----------------------------------------"
    echo ""
}

# Function to add T-Mobile Orange contract
add_tm_orange_contract() {
    echo "Adding T-Mobile <-> Orange ZK Contract..."
    
    response=$(curl -s -X POST "$API_BASE/api/tenant-blockchain-update" \
      -H "Content-Type: application/json" \
      -d '{
        "network_id": "zk_contracts_demo",
        "peer_id": "operator_tmobile", 
        "timestamp": 1756295000,
        "new_blocks": [
          {
            "block_id": 1,
            "block_hash": "zk_block_tm_orange_001",
            "timestamp": 1756295001,
            "previous_hash": "genesis_zk",
            "network_id": "zk_contracts_demo",
            "transactions": [
              "{\"id\":\"zkcontract_tm_orange_001\",\"from\":\"T-Mobile\",\"to\":\"Orange\",\"amount\":0,\"timestamp\":1756295001,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: T-Mobile <-> Orange | Rate: $15/min (ENCRYPTED) | Settlement: $15750 | Sessions: 3 | ZK Proofs: [billing_correctness: verified, settlement_aggregation: verified, range_proofs: verified] | IMSI Privacy: enabled\"}}}"
            ]
          }
        ]
      }')
    
    echo "Response: $response"
    echo "✓ T-Mobile <-> Orange contract added"
    echo ""
}

# Function to add T-Mobile Vodafone contract
add_tm_vodafone_contract() {
    echo "Adding T-Mobile <-> Vodafone ZK Contract..."
    
    response=$(curl -s -X POST "$API_BASE/api/tenant-blockchain-update" \
      -H "Content-Type: application/json" \
      -d '{
        "network_id": "zk_contracts_demo",
        "peer_id": "operator_tmobile",
        "timestamp": 1756295100,
        "new_blocks": [
          {
            "block_id": 2,
            "block_hash": "zk_block_tm_vodafone_002",
            "timestamp": 1756295101,
            "previous_hash": "zk_block_tm_orange_001",
            "network_id": "zk_contracts_demo",
            "transactions": [
              "{\"id\":\"zkcontract_tm_vodafone_002\",\"from\":\"T-Mobile\",\"to\":\"Vodafone\",\"amount\":0,\"timestamp\":1756295101,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: T-Mobile <-> Vodafone | Rate: $12/min (ENCRYPTED) | Settlement: $13800 | Sessions: 2 | ZK Proofs: [billing_correctness: verified, settlement_aggregation: verified, range_proofs: verified] | IMSI Privacy: enabled\"}}}"
            ]
          }
        ]
      }')
    
    echo "Response: $response"
    echo "✓ T-Mobile <-> Vodafone contract added"
    echo ""
}

# Function to add ZK proof sessions
add_zk_sessions() {
    echo "Adding ZK Proof Sessions..."
    
    response=$(curl -s -X POST "$API_BASE/api/tenant-blockchain-update" \
      -H "Content-Type: application/json" \
      -d '{
        "network_id": "zk_contracts_demo",
        "peer_id": "operator_tmobile",
        "timestamp": 1756295200,
        "new_blocks": [
          {
            "block_id": 3,
            "block_hash": "zk_block_sessions_003", 
            "timestamp": 1756295201,
            "previous_hash": "zk_block_tm_vodafone_002",
            "network_id": "zk_contracts_demo",
            "transactions": [
              "{\"id\":\"zksession_tm_orange_001\",\"from\":\"T-Mobile\",\"to\":\"Orange\",\"amount\":900,\"timestamp\":1756295201,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Session | Contract: T-Mobile <-> Orange | IMSI: 6fe3307a (COMMITMENT) | Duration: 60min | Amount: $900 (ENCRYPTED) | Billing ZK Proof: verified | Range Proof: [0-240min] verified\"}}}",
              "{\"id\":\"zksession_tm_orange_002\",\"from\":\"T-Mobile\",\"to\":\"Orange\",\"amount\":1050,\"timestamp\":1756295202,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Session | Contract: T-Mobile <-> Orange | IMSI: b60a978d (COMMITMENT) | Duration: 70min | Amount: $1050 (ENCRYPTED) | Billing ZK Proof: verified | Range Proof: [0-240min] verified\"}}}",
              "{\"id\":\"zksession_tm_vodafone_001\",\"from\":\"T-Mobile\",\"to\":\"Vodafone\",\"amount\":1080,\"timestamp\":1756295203,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Session | Contract: T-Mobile <-> Vodafone | IMSI: 1543fa98 (COMMITMENT) | Duration: 90min | Amount: $1080 (ENCRYPTED) | Billing ZK Proof: verified | Range Proof: [0-240min] verified\"}}}"
            ]
          }
        ]
      }')
    
    echo "Response: $response"
    echo "✓ ZK Proof sessions added"
    echo "  - T-Mobile <-> Orange: 2 sessions (IMSI: 6fe3307a, b60a978d)"
    echo "  - T-Mobile <-> Vodafone: 1 session (IMSI: 1543fa98)"
    echo ""
}

# Function to add settlement
add_settlement() {
    echo "Adding ZK Proof Settlement..."
    
    response=$(curl -s -X POST "$API_BASE/api/tenant-blockchain-update" \
      -H "Content-Type: application/json" \
      -d '{
        "network_id": "zk_contracts_demo",
        "peer_id": "validator_settlement",
        "timestamp": 1756295300,
        "new_blocks": [
          {
            "block_id": 4,
            "block_hash": "zk_block_settlement_004",
            "timestamp": 1756295301,
            "previous_hash": "zk_block_sessions_003", 
            "network_id": "zk_contracts_demo",
            "transactions": [
              "{\"id\":\"zk_settlement_tm_orange_001\",\"from\":\"Validator\",\"to\":\"Settlement\",\"amount\":15750,\"timestamp\":1756295301,\"tx_type\":{\"Message\":{\"content\":\"ZK Settlement Verification | Contract: T-Mobile <-> Orange | Total: $15750 | Sessions: 3 | Settlement Aggregation Proof: VERIFIED | Billing Correctness Proof: VERIFIED | Private Data: HIDDEN | Validator Signature: validator_001_abc123\"}}}"
            ]
          }
        ]
      }')
    
    echo "Response: $response"
    echo "✓ ZK Settlement verification added"
    echo ""
}

# Function to add additional contracts
add_orange_telefonica_contract() {
    echo "Adding Orange <-> Telefonica ZK Contract..."
    
    current_timestamp=$(date +%s)
    
    response=$(curl -s -X POST "$API_BASE/api/tenant-blockchain-update" \
      -H "Content-Type: application/json" \
      -d '{
        "network_id": "zk_contracts_demo",
        "peer_id": "operator_orange",
        "timestamp": '$current_timestamp',
        "new_blocks": [
          {
            "block_id": 5,
            "block_hash": "zk_block_orange_telefonica_005",
            "timestamp": '$current_timestamp',
            "previous_hash": "zk_block_settlement_004",
            "network_id": "zk_contracts_demo",
            "transactions": [
              "{\"id\":\"zkcontract_orange_telefonica_005\",\"from\":\"Orange\",\"to\":\"Telefonica\",\"amount\":0,\"timestamp\":'$current_timestamp',\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: Orange <-> Telefonica | Rate: $18/min (ENCRYPTED) | Settlement: $0 | Sessions: 0 | ZK Proofs: [contract_creation: verified] | IMSI Privacy: enabled\"}}}"
            ]
          }
        ]
      }')
    
    echo "Response: $response"
    echo "✓ Orange <-> Telefonica contract added"
    echo ""
}

# Function to verify contracts were added
verify_contracts() {
    echo "Verifying ZK contracts were added to blockchain..."
    echo ""
    
    echo "Current blocks with ZK contracts:"
    blocks=$(curl -s "$API_BASE/api/blocks?limit=20")
    
    if command -v jq > /dev/null; then
        echo "$blocks" | jq -r '.[] | select(.network_id == "zk_contracts_demo") | "Block #\(.block_id): \(.block_hash) - \(.transactions | length) transactions"'
        echo ""
        echo "ZK Contract transactions:"
        echo "$blocks" | jq -r '.[] | select(.network_id == "zk_contracts_demo") | .transactions[] | fromjson | "- \(.id): \(.from) -> \(.to) (\(.amount) amount)"'
    else
        echo "Install jq for better formatting. Raw response:"
        echo "$blocks"
    fi
    echo ""
}

# Function to test ZK proof API
test_zk_api() {
    echo "Testing ZK Proof API endpoints..."
    
    echo "T-Mobile contracts (should see both decrypted):"
    curl -s "$API_BASE/api/operator-contracts?operator=tmobile" | jq . 2>/dev/null || echo "Raw response: $(curl -s "$API_BASE/api/operator-contracts?operator=tmobile")"
    echo ""
    
    echo "Orange contracts (should see T-Mobile<->Orange decrypted only):"
    curl -s "$API_BASE/api/operator-contracts?operator=orange" | jq . 2>/dev/null || echo "Raw response: $(curl -s "$API_BASE/api/operator-contracts?operator=orange")"
    echo ""
    
    echo "AT&T contracts (should see all encrypted):"
    curl -s "$API_BASE/api/operator-contracts?operator=att" | jq . 2>/dev/null || echo "Raw response: $(curl -s "$API_BASE/api/operator-contracts?operator=att")"
    echo ""
}

# Function to show dashboard URLs
show_dashboards() {
    echo "=========================================="
    echo "ZK Proof Dashboards"
    echo "=========================================="
    echo ""
    echo "View your ZK proof contracts in the web dashboard:"
    echo ""
    echo "🌐 Enterprise Dashboard: $DASHBOARD_BASE"
    echo "   - Shows blockchain blocks with ZK transactions"
    echo "   - View transaction details and messages"
    echo ""
    echo "🔐 ZK Proof Dashboard: $DASHBOARD_BASE/zk"  
    echo "   - Role-based operator views"
    echo "   - Switch between T-Mobile, Orange, Vodafone, AT&T, Validator"
    echo "   - See encrypted vs decrypted contract data"
    echo ""
    echo "📊 Quick Links:"
    echo "   - Main Dashboard: $DASHBOARD_BASE"
    echo "   - ZK Dashboard: $DASHBOARD_BASE/zk"
    echo "   - API Health: $API_BASE/health"
    echo "   - API Status: $API_BASE/api/status"
    echo "   - Recent Blocks: $API_BASE/api/blocks?limit=10"
    echo ""
}

# Main execution
main() {
    case "${1:-all}" in
        "check")
            check_api
            check_blockchain_status
            ;;
        "contracts")
            check_api
            add_tm_orange_contract
            add_tm_vodafone_contract
            add_orange_telefonica_contract
            ;;
        "sessions")
            check_api
            add_zk_sessions
            ;;
        "settlement")
            check_api
            add_settlement
            ;;
        "verify")
            check_api
            verify_contracts
            ;;
        "test")
            check_api
            test_zk_api
            ;;
        "clean")
            echo "To clean blockchain data, stop the validator and delete:"
            echo "  rm data/enterprise_blockchain_validator1.json"
            echo "Then restart the validator."
            ;;
        "all")
            check_api
            echo "STEP 1: Current blockchain status"
            check_blockchain_status
            
            echo "STEP 2: Adding ZK contracts"
            add_tm_orange_contract
            add_tm_vodafone_contract
            
            echo "STEP 3: Adding ZK sessions"
            add_zk_sessions
            
            echo "STEP 4: Adding settlement"
            add_settlement
            
            echo "STEP 5: Adding additional contract"
            add_orange_telefonica_contract
            
            echo "STEP 6: Verifying contracts"
            verify_contracts
            
            echo "STEP 7: Testing ZK API"
            test_zk_api
            
            show_dashboards
            ;;
        "help"|*)
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  all        - Run complete demo (default)"
            echo "  check      - Check API and blockchain status"
            echo "  contracts  - Add ZK proof contracts only"
            echo "  sessions   - Add ZK proof sessions only"
            echo "  settlement - Add ZK settlement only"
            echo "  verify     - Verify contracts in blockchain"
            echo "  test       - Test ZK proof API endpoints"
            echo "  clean      - Show how to clean blockchain data"
            echo "  help       - Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run complete demo"
            echo "  $0 check              # Check current status"
            echo "  $0 contracts          # Add contracts only"
            echo "  $0 verify             # Verify what's in blockchain"
            echo ""
            ;;
    esac
}

# Run main function with all arguments
main "$@"

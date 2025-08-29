#!/bin/bash

echo "🔐 ZK Proof Contracts Setup Script"
echo "=================================="
echo ""

# Check API health
echo "📡 Checking API health..."
curl -s http://192.168.200.133:8080/health
echo -e "\n\n✅ Press any key to create T-Mobile <-> Orange contract..."
read -n 1 -s

# T-Mobile <-> Orange Contract
echo -e "\n📝 Creating T-Mobile <-> Orange contract..."
curl -X POST http://192.168.200.133:8080/api/tenant-blockchain-update -H "Content-Type: application/json" -d '{"network_id":"zk_contracts_demo","peer_id":"operator_tmobile","timestamp":1756295000,"new_blocks":[{"block_id":1,"block_hash":"zk_block_tm_orange_001","timestamp":1756295001,"previous_hash":"genesis_zk","network_id":"zk_contracts_demo","transactions":["{\"id\":\"zkcontract_tm_orange_001\",\"from\":\"T-Mobile\",\"to\":\"Orange\",\"amount\":0,\"timestamp\":1756295001,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: T-Mobile <-> Orange | Rate: $15/min (ENCRYPTED) | Settlement: $15750 | Sessions: 3 | ZK Proofs: [billing_correctness: verified, settlement_aggregation: verified, range_proofs: verified] | IMSI Privacy: enabled\"}}}"]}]}'
echo -e "\n\n✅ Press any key to create T-Mobile <-> Vodafone contract..."
read -n 1 -s

# T-Mobile <-> Vodafone Contract
echo -e "\n📝 Creating T-Mobile <-> Vodafone contract..."
curl -X POST http://192.168.200.133:8080/api/tenant-blockchain-update -H "Content-Type: application/json" -d '{"network_id":"zk_contracts_demo","peer_id":"operator_tmobile","timestamp":1756295100,"new_blocks":[{"block_id":2,"block_hash":"zk_block_tm_vodafone_002","timestamp":1756295101,"previous_hash":"zk_block_tm_orange_001","network_id":"zk_contracts_demo","transactions":["{\"id\":\"zkcontract_tm_vodafone_002\",\"from\":\"T-Mobile\",\"to\":\"Vodafone\",\"amount\":0,\"timestamp\":1756295101,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: T-Mobile <-> Vodafone | Rate: $12/min (ENCRYPTED) | Settlement: $13800 | Sessions: 2 | ZK Proofs: [billing_correctness: verified, settlement_aggregation: verified, range_proofs: verified] | IMSI Privacy: enabled\"}}}"]}]}'
echo -e "\n\n✅ Press any key to create AT&T <-> Vodafone contract..."
read -n 1 -s

# AT&T <-> Vodafone Contract
echo -e "\n📝 Creating AT&T <-> Vodafone contract..."
curl -X POST http://192.168.200.133:8080/api/tenant-blockchain-update -H "Content-Type: application/json" -d '{"network_id":"zk_contracts_demo","peer_id":"operator_att","timestamp":1756295150,"new_blocks":[{"block_id":3,"block_hash":"zk_block_att_vodafone_003","timestamp":1756295151,"previous_hash":"zk_block_tm_vodafone_002","network_id":"zk_contracts_demo","transactions":["{\"id\":\"zkcontract_att_vodafone_003\",\"from\":\"AT&T\",\"to\":\"Vodafone\",\"amount\":0,\"timestamp\":1756295151,\"tx_type\":{\"Message\":{\"content\":\"ZK Private Contract | Parties: AT&T <-> Vodafone | Rate: $18/min (ENCRYPTED) | Settlement: $21600 | Sessions: 4 | ZK Proofs: [billing_correctness: verified, settlement_aggregation: verified, range_proofs: verified] | IMSI Privacy: enabled\"}}}"]}]}'
echo -e "\n\n✅ Press any key to verify contracts were added..."
read -n 1 -s

# Verify contracts
echo -e "\n🔍 Verifying contracts in blockchain..."
curl -s http://192.168.200.133:8080/api/blocks?limit=20 | grep "zk_contracts_demo"
echo -e "\n\n✅ Press any key to check T-Mobile's view..."
read -n 1 -s

# Check T-Mobile view
echo -e "\n👤 T-Mobile's view (can see Orange & Vodafone contracts):"
curl -s http://192.168.200.133:8080/api/operator-contracts?operator=tmobile | python3 -m json.tool
echo -e "\n✅ Press any key to check Vodafone's view..."
read -n 1 -s

# Check Vodafone view
echo -e "\n👤 Vodafone's view (can see T-Mobile & AT&T contracts):"
curl -s http://192.168.200.133:8080/api/operator-contracts?operator=vodafone | python3 -m json.tool
echo -e "\n✅ Press any key to check AT&T's view..."
read -n 1 -s

# Check AT&T view
echo -e "\n👤 AT&T's view (can only see Vodafone contract):"
curl -s http://192.168.200.133:8080/api/operator-contracts?operator=att | python3 -m json.tool
echo -e "\n✅ Press any key to check Orange's view..."
read -n 1 -s

# Check Orange view
echo -e "\n👤 Orange's view (can only see T-Mobile contract):"
curl -s http://192.168.200.133:8080/api/operator-contracts?operator=orange | python3 -m json.tool

echo -e "\n\n🎉 Setup Complete!"
echo "📊 View the dashboard at: http://192.168.200.133:9090/zk"
echo ""
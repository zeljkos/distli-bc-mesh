#!/bin/bash

# Multi-Country Telecom Operator Demo
# Simulates different countries running their own key management systems

echo "Starting Multi-Country Telecom ZK Demo"
echo "====================================="

# Kill any existing processes
pkill -f "enterprise_dashboard"

echo "Creating country-specific blockchain data..."

# Create separate blockchain files for each country
mkdir -p data/countries

# USA - T-Mobile data
cat > data/countries/usa_blockchain.json << 'EOF'
{
  "country": "USA",
  "operator": "T-Mobile",
  "jurisdiction": "FCC",
  "key_storage": "US_HSM_DATACENTER",
  "private_key_location": "Bellevue_WA_Secure_Facility",
  "chain": [
    {
      "hash": "usa_genesis_block",
      "height": 0,
      "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
      "stake_weight": 0,
      "timestamp": 1724932800,
      "transactions": [],
      "validator": "tmobile_usa_validator"
    }
  ],
  "contracts": {},
  "pending": [],
  "tenant_blocks": [],
  "validators": {
    "tmobile_validator": {
      "active": true,
      "address": "tmobile_usa_hsm_cluster_1",
      "stake": 1000000,
      "last_block_time": 1724932800,
      "jurisdiction": "USA_FCC",
      "key_escrow": "NSA_COMPLIANT"
    }
  }
}
EOF

# France - Orange data  
cat > data/countries/france_blockchain.json << 'EOF'
{
  "country": "France",
  "operator": "Orange",
  "jurisdiction": "ARCEP",
  "key_storage": "EU_HSM_DATACENTER",
  "private_key_location": "Paris_Secure_Facility",
  "chain": [
    {
      "hash": "france_genesis_block",
      "height": 0,
      "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
      "stake_weight": 0,
      "timestamp": 1724932800,
      "transactions": [],
      "validator": "orange_france_validator"
    }
  ],
  "contracts": {},
  "pending": [],
  "tenant_blocks": [],
  "validators": {
    "orange_validator": {
      "active": true,
      "address": "orange_france_hsm_cluster_1", 
      "stake": 1000000,
      "last_block_time": 1724932800,
      "jurisdiction": "FRANCE_ARCEP",
      "key_escrow": "GDPR_COMPLIANT"
    }
  }
}
EOF

# Germany - Deutsche Telekom data
cat > data/countries/germany_blockchain.json << 'EOF'
{
  "country": "Germany", 
  "operator": "Deutsche_Telekom",
  "jurisdiction": "BNetzA",
  "key_storage": "DE_HSM_DATACENTER",
  "private_key_location": "Bonn_Secure_Facility",
  "chain": [
    {
      "hash": "germany_genesis_block",
      "height": 0,
      "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
      "stake_weight": 0,
      "timestamp": 1724932800,
      "transactions": [],
      "validator": "dt_germany_validator"
    }
  ],
  "contracts": {},
  "pending": [],
  "tenant_blocks": [],
  "validators": {
    "dt_validator": {
      "active": true,
      "address": "dt_germany_hsm_cluster_1",
      "stake": 1000000,
      "last_block_time": 1724932800,
      "jurisdiction": "GERMANY_BNetzA", 
      "key_escrow": "EU_PRIVACY_COMPLIANT"
    }
  }
}
EOF

# UK - Vodafone data
cat > data/countries/uk_blockchain.json << 'EOF'
{
  "country": "UK",
  "operator": "Vodafone",
  "jurisdiction": "Ofcom",
  "key_storage": "UK_HSM_DATACENTER", 
  "private_key_location": "London_Secure_Facility",
  "chain": [
    {
      "hash": "uk_genesis_block",
      "height": 0,
      "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
      "stake_weight": 0,
      "timestamp": 1724932800,
      "transactions": [],
      "validator": "vodafone_uk_validator"
    }
  ],
  "contracts": {},
  "pending": [],
  "tenant_blocks": [],
  "validators": {
    "vodafone_validator": {
      "active": true,
      "address": "vodafone_uk_hsm_cluster_1",
      "stake": 1000000,
      "last_block_time": 1724932800,
      "jurisdiction": "UK_OFCOM",
      "key_escrow": "POST_BREXIT_COMPLIANT"
    }
  }
}
EOF

echo "Starting country-specific dashboards..."

# Start dashboards for each country
cargo build --release

echo "Starting USA (T-Mobile) dashboard on port 9000..."
RUST_LOG=info ./target/release/enterprise_dashboard 9000 data/countries/usa_blockchain.json &
sleep 2

echo "Starting France (Orange) dashboard on port 9001..."
RUST_LOG=info ./target/release/enterprise_dashboard 9001 data/countries/france_blockchain.json &
sleep 2

echo "Starting Germany (Deutsche Telekom) dashboard on port 9002..." 
RUST_LOG=info ./target/release/enterprise_dashboard 9002 data/countries/germany_blockchain.json &
sleep 2

echo "Starting UK (Vodafone) dashboard on port 9003..."
RUST_LOG=info ./target/release/enterprise_dashboard 9003 data/countries/uk_blockchain.json &
sleep 2

echo ""
echo "Multi-Country Demo Ready!"
echo "========================"
echo ""
echo "Access dashboards:"
echo "USA (T-Mobile):        http://192.168.200.133:9000"
echo "France (Orange):       http://192.168.200.133:9001"  
echo "Germany (DT):          http://192.168.200.133:9002"
echo "UK (Vodafone):         http://192.168.200.133:9003"
echo ""
echo "ZK Privacy Demos:"
echo "USA ZK Dashboard:      http://192.168.200.133:9000/zk"
echo "France ZK Dashboard:   http://192.168.200.133:9001/zk"
echo "Germany ZK Dashboard:  http://192.168.200.133:9002/zk"
echo "UK ZK Dashboard:       http://192.168.200.133:9003/zk"
echo ""
echo "Demo Scenarios:"
echo "==============="
echo "1. T-Mobile subscriber roams in France (Orange network)"
echo "   - T-Mobile can decrypt their subscriber data"
echo "   - Orange sees encrypted IMSI commitments only"
echo ""
echo "2. Orange subscriber roams in Germany (DT network)"  
echo "   - Orange can decrypt their subscriber data"
echo "   - Deutsche Telekom sees encrypted IMSI commitments only"
echo ""
echo "3. Cross-operator billing settlement"
echo "   - Each operator decrypts only their own customers"
echo "   - ZK proofs verify billing without revealing call details"
echo ""
echo "Key Management Simulation:"
echo "========================="
echo "- Each country stores private keys locally"
echo "- Public keys shared globally via blockchain"
echo "- Jurisdictional compliance simulated per country"
echo "- No single operator can decrypt all traffic"
echo ""
echo "Press Ctrl+C to stop all dashboards"

# Wait for user to stop
trap 'echo "Stopping all dashboards..."; pkill -f "enterprise_dashboard"; exit' INT
wait
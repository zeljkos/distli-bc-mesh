
● Multi-Country ZK Demo - Complete Guide

  Quick Start Commands:

  # 1. Create ZK blockchain data with real proofs
  ./create_live_zk_data_fixed.sh

  # 2. Start multi-country operator dashboards
  ./start_multi_country_demo.sh

  What Each Script Does:

  create_live_zk_data_fixed.sh:

  - Runs real ZK proof demonstrations (cargo run --example zk_range_proof_demo)
  - Creates blockchain data with real cryptographic commitments
  - Generates T-Mobile ↔ Orange, T-Mobile ↔ Vodafone contracts
  - Creates voice calls, SMS, and data sessions with 672-byte Bulletproof ZK proofs
  - Updates data/enterprise_blockchain_validator1.json with ZK data
  - Result: Your existing validator (port 8080) picks up real ZK data automatically

  start_multi_country_demo.sh:

  - Creates country-specific blockchain files in data/countries/
  - Starts 4 separate dashboards simulating different jurisdictions:
    - 9000 = USA (T-Mobile HQ) - US compliance
    - 9001 = France (Orange HQ) - EU GDPR compliance
    - 9002 = Germany (Deutsche Telekom HQ) - German telecom law
    - 9003 = UK (Vodafone HQ) - Post-Brexit compliance
  - Each "country" has isolated private key management
  - Demonstrates jurisdictional separation without VMs/VPNs

  Demo URLs:

  Original Validator (with ZK data):
  - Main: http://192.168.200.133:8080
  - ZK Dashboard: http://192.168.200.133:8080/zk

  Multi-Country Simulation:
  - USA: http://192.168.200.133:9000/zk
  - France: http://192.168.200.133:9001/zk
  - Germany: http://192.168.200.133:9002/zk
  - UK: http://192.168.200.133:9003/zk

  Demo Scenarios:

  1. Privacy Protection: T-Mobile subscriber roams in France → only T-Mobile can decrypt IMSI
  2. Cross-Border Billing: Orange subscriber roams in Germany → only Orange can bill their customer
  3. Regulatory Compliance: Each country controls their operators' private keys
  4. ZK Proof Verification: All sessions show "VERIFIED" with 672-byte proofs

  Files Created:

  - Real ZK blockchain data → data/enterprise_blockchain_validator1.json
  - Country-specific data → data/countries/{usa,france,germany,uk}_blockchain.json
  - Multi-country demo script → start_multi_country_demo.sh
  - Updated dashboard → src/enterprise_bc/dashboard.rs (country-aware)

  This gives you both real cryptographic ZK proofs and multi-jurisdictional key management simulation without complex VM setups.


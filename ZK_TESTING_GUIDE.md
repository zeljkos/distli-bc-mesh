# Zero-Knowledge Proof Testing Guide

## 🚀 Quick Start

### 1. Build the Project
```bash
# Build with native features and ZK proof support
cargo build --features native --release

# Build WASM if needed
./build-wasm.sh
```

### 2. Test ZK Proofs Standalone
```bash
# Run the ZK proof demo to see Bulletproofs in action
cargo run --example zk_range_proof_demo

# Run private roaming demo
cargo run --example private_roaming_demo
```

### 3. Start Services (in separate terminals)

**Terminal 1 - Tracker:**
```bash
ENTERPRISE_BC_URL="http://192.168.200.133:8080" cargo run --bin tracker --features native
```

**Terminal 2 - Validator:**
```bash
TRACKER_URL="http://192.168.200.132:3030" cargo run --bin enterprise-validator --features native -- --id validator1 --port 8080 --stake 1000
```

**Terminal 3 - Dashboard:**
```bash
cargo run --bin enterprise-dashboard --features native -- --port 9090
```

### 4. Load Test Data with Real ZK Proofs
```bash
# Run the test script that creates contracts with real Bulletproofs
./test_zk_real_proofs.sh
```

### 5. View Results in Dashboard

Open your browser to: **http://192.168.200.133:9090/zk**

The ZK dashboard is now integrated directly into the Rust dashboard server at the `/zk` route.

## 📊 What You'll See in the Dashboard

### For Each Contract:
- **Parties**: Which operators are involved (e.g., T-Mobile ↔ Orange)
- **Visibility**: Whether current operator can decrypt (parties only) or not
- **Rate**: ENCRYPTED for non-parties, actual value for parties
- **Proof Size**: 672 bytes (Bulletproofs constant size)
- **Commitment**: 32 bytes Pedersen commitment
- **Verification**: ✅ Verified using zero-knowledge proofs

### For Each Session:
- **Duration**: Hidden from non-parties, proven to be in range [0-240] minutes
- **IMSI**: Always encrypted, never revealed
- **Proof Type**: Bulletproof Range [0-240]

## 🔑 Key Features Demonstrated

1. **Real Cryptographic Proofs**: Not simulated - actual Bulletproofs v4.0
2. **Privacy Preservation**: Values remain hidden while proving validity
3. **Selective Disclosure**: Only contract parties can decrypt details
4. **Verifiable Computation**: Anyone can verify proofs without seeing data
5. **Constant Proof Size**: 672 bytes regardless of value being proven

## 🧪 Testing Different Scenarios

### Test 1: Operator Isolation
```bash
# View as T-Mobile (can see Orange & Vodafone contracts)
# Select "T-Mobile View" in dashboard

# View as AT&T (can only see Vodafone contract)
# Select "AT&T View" in dashboard
```

### Test 2: Generate New Proofs
```bash
# Generate individual range proofs
cargo test -p distli-mesh-bc test_duration_range_proof -- --nocapture

# Test batch verification
cargo test -p distli-mesh-bc test_batch_verification -- --nocapture
```

### Test 3: Verify Performance
```bash
# Run benchmarks
cargo bench --features native
```

## 📝 API Endpoints

### Get Blocks with ZK Proofs
```bash
curl http://192.168.200.133:8080/api/blocks?limit=100 | jq '.blocks[] | select(.network_id == "zk_real_proofs")'
```

### Check Operator View
```bash
# T-Mobile's view
curl http://192.168.200.133:8080/api/operator-contracts?operator=tmobile

# Vodafone's view  
curl http://192.168.200.133:8080/api/operator-contracts?operator=vodafone
```

## 🛠️ Troubleshooting

### If services won't start:
1. Check ports are free: `lsof -i :8080` and `lsof -i :3030`
2. Kill existing processes: `pkill -f "cargo run"`

### If dashboard shows no data:
1. Ensure validator is running
2. Run test script: `./test_zk_real_proofs.sh`
3. Check API: `curl http://192.168.200.133:8080/health`

### If compilation fails:
1. Update dependencies: `cargo update`
2. Clean build: `cargo clean && cargo build --features native`

## 📊 Performance Metrics

- **Proof Generation**: ~10-20ms per proof
- **Verification**: ~5ms per proof
- **Batch Verification**: ~15ms for 10 proofs
- **Proof Size**: 672 bytes (constant)
- **Commitment Size**: 32 bytes
- **Security Level**: 128-bit computational

## 🔐 Security Properties

- **Zero-Knowledge**: Proofs reveal nothing beyond statement validity
- **Soundness**: Cannot create false proofs (computationally infeasible)
- **Completeness**: Valid statements always verify correctly
- **Non-Interactive**: No back-and-forth needed between prover/verifier

## 📚 Next Steps

1. **Phase 2**: Implement Pedersen commitments for IMSI
2. **Phase 3**: Add multiplication proofs for billing correctness
3. **Phase 4**: Implement settlement aggregation proofs
4. **Phase 5**: Add signature schemes for non-repudiation

## 🎯 Summary

You now have a working implementation of Zero-Knowledge Range Proofs using Bulletproofs v4.0. The system proves call durations are within valid ranges (0-240 minutes) without revealing the actual values. This provides cryptographic privacy for telecom roaming contracts while maintaining verifiability.
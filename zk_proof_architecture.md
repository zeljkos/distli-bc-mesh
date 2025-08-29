# Zero-Knowledge Proof Architecture in Telecom Roaming

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     🔐 ZERO-KNOWLEDGE PROOF SYSTEM                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                            📋 CONTRACT CREATION                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  T-Mobile                                        Orange                     │
│  ┌─────────┐                                    ┌─────────┐                │
│  │ Rate:   │                                    │ Rate:   │                │
│  │ $15/min │ ──────┐                   ┌──────── │ $15/min │                │
│  │ (clear) │       │                   │        │ (clear) │                │
│  └─────────┘       │                   │        └─────────┘                │
│                    │                   │                                   │
│                    ▼                   ▼                                   │
│              ┌─────────────────────────────────┐                           │
│              │   🔒 ENCRYPTION LAYER           │                           │
│              │                                 │                           │
│              │  Rate: ENCRYPTED                │                           │
│              │  Proof: 672_bytes_bulletproof   │                           │
│              │  Commitment: 32_bytes           │                           │
│              │  Range: [0,240] minutes         │                           │
│              └─────────────────────────────────┘                           │
│                            │                                               │
│                            ▼                                               │
│              ┌─────────────────────────────────┐                           │
│              │    📡 BLOCKCHAIN STORAGE        │                           │
│              │                                 │                           │
│              │  Public: Contract exists        │                           │
│              │  Private: Rate encrypted        │                           │
│              │  ZK Proof: Provably valid       │                           │
│              └─────────────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           📊 USAGE SESSIONS                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Subscriber roams from T-Mobile to Orange network                          │
│                                                                             │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐                │
│  │ Session:    │      │ ZK Proof    │      │ Blockchain  │                │
│  │ Duration:   │ ──── │ Generator   │ ──── │ Storage     │                │
│  │ 45 minutes  │      │             │      │             │                │
│  │ (actual)    │      │ Bulletproof │      │ Duration:   │                │
│  └─────────────┘      │ Range Proof │      │ ENCRYPTED   │                │
│                       │             │      │ Proof: 672B │                │
│                       │ Proves:     │      │ Valid: ✓    │                │
│                       │ 0 ≤ x ≤ 240 │      │             │                │
│                       │ (minutes)   │      │ IMSI:       │                │
│                       │             │      │ ENCRYPTED   │                │
│                       │ Reveals:    │      │             │                │
│                       │ NOTHING!    │      └─────────────┘                │
│                       │ about actual│                                       │
│                       │ duration    │                                       │
│                       └─────────────┘                                       │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                          🔍 VIEWING PERMISSIONS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  T-Mobile View (Participant)          │  Validator View (Non-participant)  │
│  ┌─────────────────────────────────┐   │  ┌─────────────────────────────────┐│
│  │ ✅ CAN DECRYPT:                 │   │  │ ❌ CANNOT DECRYPT:              ││
│  │                                 │   │  │                                 ││
│  │ Rate: $15/min (decrypted)       │   │  │ Rate: ENCRYPTED                 ││
│  │ Sessions: 45, 60, 75 min        │   │  │ Sessions: ENCRYPTED             ││
│  │ Settlement: $15,750             │   │  │ Settlement: [public amount]     ││
│  │ IMSI: [committed, not revealed] │   │  │ IMSI: ENCRYPTED                 ││
│  │                                 │   │  │                                 ││
│  │ ✅ CAN VERIFY:                  │   │  │ ✅ CAN VERIFY:                  ││
│  │ - All ZK proofs are valid       │   │  │ - All ZK proofs are valid       ││
│  │ - Durations are in range        │   │  │ - Durations are in range        ││
│  │ - Billing calculations correct  │   │  │ - Billing calculations correct  ││
│  └─────────────────────────────────┘   │  └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                          🧮 CRYPTOGRAPHIC COMPONENTS                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Bulletproofs Library (Curve25519-dalek)                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  📝 COMMITMENT PHASE:                                               │   │
│  │  ┌─────────────────┐    ┌─────────────────┐                        │   │
│  │  │ Secret Value:   │    │ Random Blinding │                        │   │
│  │  │ duration = 45   │ +  │ factor = r      │ → Commitment = g^45·h^r│   │
│  │  │ (hidden)        │    │ (hidden)        │   (public)             │   │
│  │  └─────────────────┘    └─────────────────┘                        │   │
│  │                                                                     │   │
│  │  🔢 RANGE PROOF:                                                    │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │ Proves: "The committed value is between 0 and 240"         │   │   │
│  │  │ Without revealing: What the actual value is                │   │   │
│  │  │ Size: Exactly 672 bytes (constant, regardless of value)   │   │   │
│  │  │ Time: ~5ms to verify                                       │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  │  ✅ VERIFICATION:                                                   │   │
│  │  Anyone can verify the proof is valid without learning the secret  │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                             🔄 DATA FLOW                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Contract Creation:                                                      │
│     T-Mobile ←→ Orange: Rate $15/min → Encrypt → Store as "ENCRYPTED"      │
│                                                                             │
│  2. Session Recording:                                                      │
│     Actual: 45 min → ZK Proof → Store: DURATION:45|PROOF:672B|VALID:true   │
│                                                                             │
│  3. Dashboard Access:                                                       │
│     ┌─────────────┐    ┌──────────────┐    ┌─────────────┐                │
│     │ T-Mobile    │    │ Decrypt      │    │ Show:       │                │
│     │ Login       │ ── │ Function     │ ── │ $15/min     │                │
│     │ (authorized)│    │ (has keys)   │    │ (decrypted) │                │
│     └─────────────┘    └──────────────┘    └─────────────┘                │
│                                                                             │
│     ┌─────────────┐    ┌──────────────┐    ┌─────────────┐                │
│     │ Validator   │    │ No Keys      │    │ Show:       │                │
│     │ Login       │ ── │ Cannot       │ ── │ ENCRYPTED   │                │
│     │ (observer)  │    │ Decrypt      │    │ (hidden)    │                │
│     └─────────────┘    └──────────────┘    └─────────────┘                │
│                                                                             │
│  4. Proof Verification (Available to All):                                 │
│     ✅ Duration proof valid                                                 │
│     ✅ Range [0,240] minutes satisfied                                      │
│     ✅ Billing calculation correct                                          │
│     ✅ Settlement aggregation valid                                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           🛡️ SECURITY PROPERTIES                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅ PRIVACY:                                                                │
│     - Actual rates hidden from competitors                                  │
│     - Session details encrypted                                             │
│     - Only participants can decrypt their contracts                        │
│                                                                             │
│  ✅ INTEGRITY:                                                              │
│     - All proofs cryptographically verifiable                              │
│     - Tampering detected immediately                                        │
│     - Settlement calculations provably correct                              │
│                                                                             │
│  ✅ NON-REPUDIATION:                                                        │
│     - All actions recorded on blockchain                                    │
│     - ZK proofs provide undeniable evidence                                 │
│     - Disputes can be resolved without revealing secrets                    │
│                                                                             │
│  ✅ REGULATORY COMPLIANCE:                                                  │
│     - Validators can verify all transactions are legitimate                 │
│     - No sensitive data exposed to unauthorized parties                     │
│     - Audit trails maintained without compromising privacy                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Files in the Implementation:

**Core ZK Implementation:**
- `src/common/zk_range_proofs.rs` - Bulletproof range proof generation/verification
- `src/common/private_contracts.rs` - Private contract management with ZK integration

**Dashboard & API:**
- `src/enterprise_bc/dashboard.rs` - Web interface with operator-specific decryption 
- `src/enterprise_bc/api.rs` - REST API endpoints for ZK data access

**Test Data:**
- `data/tenant_zk_real_proofs.json` - Real Bulletproof ZK contracts on blockchain
- `examples/zk_range_proof_demo.rs` - Demo showing ZK proof generation

The system uses **real Bulletproof cryptography** (not simulation) to provide mathematical guarantees of privacy while enabling public verification of billing correctness.
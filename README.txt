## Persistent Storage

### Enterprise Blockchain
- **Location**: `data/enterprise_blockchain_{validator_id}.json`
- **Contains**: Complete blockchain state, pending updates, validator information
- **Auto-saves**: On every block addition and state change

### Tracker Integration  
- **Location**: `data/tracker_integration.json`
- **Contains**: Network states, reported data, blockchain summaries
- **Auto-saves**: On every enterprise report

### Web Client
- **Location**: Browser memory (session-based per network)
- **Contains**: Per-network blockchain state and transactions
- **Auto-saves**: On every transaction and block mining





# Migration Guide: Restructuring distli-mesh-bc

## 🎯 Goal: Single Project with Tracker + Enterprise BC

Transform your existing `distli-mesh-bc` into a unified project with maximum code reuse.

## 📂 Current vs New Structure

### **Before (Your Current Structure):**
```
distli-mesh-bc/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── blockchain.rs
│   ├── tracker.rs
│   └── types.rs
└── public/
    └── index.html
```

### **After (New Unified Structure):**
```
distli-mesh-bc/
├── Cargo.toml               # ← Updated with multiple binaries
├── src/
│   ├── lib.rs               # ← NEW: Shared library
│   ├── common/              # ← NEW: Shared code
│   ├── tracker/             # ← MOVED: Existing tracker code
│   └── enterprise_bc/       # ← NEW: Enterprise blockchain
├── public/                  # ← Same as before
└── docker/                  # ← NEW: Docker setup
```

## 🚀 Step-by-Step Migration

### **Step 1: Backup Your Current Work**
```bash
# In your distli-mesh-bc directory
git add .
git commit -m "Backup before restructuring"
git tag backup-before-restructure
```

### **Step 2: Create New Directory Structure**
```bash
# Create new directories
mkdir -p src/common src/tracker src/enterprise_bc docker

# Create module files
touch src/lib.rs
touch src/common/mod.rs
touch src/tracker/mod.rs  
touch src/enterprise_bc/mod.rs
```

### **Step 3: Move Existing Files**
```bash
# Move existing files to new locations
mv src/main.rs src/tracker/main.rs
mv src/tracker.rs src/tracker/server.rs
mv src/types.rs src/common/types.rs
mv src/blockchain.rs src/common/blockchain.rs

# Keep public/ as-is (no changes needed)
```

### **Step 4: Replace Cargo.toml**
Replace your existing `Cargo.toml` with the multi-binary version:

```toml
[package]
name = "distli-mesh-bc"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "tracker"
path = "src/tracker/main.rs"

[[bin]]
name = "enterprise-validator"
path = "src/enterprise_bc/validator_main.rs"

[[bin]]
name = "enterprise-dashboard"
path = "src/enterprise_bc/dashboard_main.rs"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
futures-util = "0.3"
tokio-stream = "0.1"
sha2 = "0.10"
hex = "0.4"
dashmap = "5.4"
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4.0", features = ["derive"] }
```

### **Step 5: Create All New Files**
Copy all the files I provided into the correct locations:

**Common modules:**
- `src/lib.rs`
- `src/common/mod.rs` 
- `src/common/blockchain.rs`
- `src/common/types.rs`
- `src/common/crypto.rs`
- `src/common/time.rs`
- `src/common/api_utils.rs`

**Tracker modules:**
- `src/tracker/mod.rs`
- `src/tracker/main.rs`
- `src/tracker/server.rs` (update your existing tracker logic)
- `src/tracker/integration.rs`

**Enterprise BC modules:**
- `src/enterprise_bc/mod.rs`
- `src/enterprise_bc/validator_main.rs`
- `src/enterprise_bc/dashboard_main.rs`
- `src/enterprise_bc/blockchain.rs`
- `src/enterprise_bc/validator.rs`
- `src/enterprise_bc/consensus.rs`
- `src/enterprise_bc/api.rs`
- `src/enterprise_bc/dashboard.rs`

**Docker setup:**
- `docker/docker-compose.yml`
- `docker/Dockerfile`
- `docker/nginx.conf`
- `docker/setup.sh`

### **Step 6: Update Import Statements**
Update your existing code to use the new module structure:

**In tracker files, change:**
```rust
// OLD
use crate::types::Message;
use crate::blockchain::Block;

// NEW  
use crate::common::types::Message;
use crate::common::blockchain::Block;
```

### **Step 7: Test the Migration**
```bash
# Test tracker still works
cargo run --bin tracker

# Test enterprise validator builds
cargo run --bin enterprise-validator -- --help

# Test dashboard builds
cargo run --bin enterprise-dashboard -- --help
```

## 🔧 Quick Setup Commands

### **For Local Development:**
```bash
# Run tracker (your existing functionality)
cargo run --bin tracker

# Run single validator for testing
cargo run --bin enterprise-validator -- --id test-validator

# Run dashboard
cargo run --bin enterprise-dashboard
```

### **For Production (Docker on 192.168.200.133):**
```bash
# Copy entire project to VM
scp -r distli-mesh-bc/ user@192.168.200.133:~/

# On the VM
cd distli-mesh-bc/docker
chmod +x setup.sh
./setup.sh
```

### **With Enterprise Integration:**
```bash
# On tracker machine (192.168.200.132)
ENTERPRISE_BC_URL="http://192.168.200.133:8080" cargo run --bin tracker
```

## 🎯 Benefits After Migration

✅ **Existing tracker works exactly the same**  
✅ **New enterprise blockchain system added**  
✅ **Maximum code reuse between systems**  
✅ **Single git repository for everything**  
✅ **Docker support for enterprise deployment**  
✅ **Automatic integration between tracker and enterprise BC**  

## 🆘 If Something Goes Wrong

```bash
# Revert to backup
git reset --hard backup-before-restructure

# Start over with the migration
```

## 📋 Final Verification

After migration, you should be able to:

1. ✅ Run tracker: `cargo run --bin tracker`
2. ✅ See your existing web UI at `http://localhost:3030`
3. ✅ Create networks and connect peers (same as before)
4. ✅ Run enterprise BC: `cd docker && ./setup.sh`
5. ✅ See enterprise dashboard at `http://192.168.200.133:9090`
6. ✅ See tenant data flowing from tracker to enterprise BC

The migration preserves all your existing functionality while adding the enterprise blockchain system!

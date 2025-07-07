#!/bin/bash

echo "🔧 Fixing WASM build issues..."

# Fix 1: Make common module conditional in lib.rs
echo "📝 Fixing src/lib.rs..."
cat > src/lib.rs << 'EOF'
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod blockchain;

// Only include common module in native builds to avoid server dependencies in WASM
#[cfg(feature = "native")]
pub mod common;

#[cfg(feature = "native")]
pub mod tracker;

#[cfg(feature = "native")]
pub mod enterprise_bc;

// Re-export main types
pub use blockchain::{Blockchain, OrderBook, Block, Transaction, TransactionType};

// Enterprise types only for native
#[cfg(feature = "native")]
pub use blockchain::{TenantBlockchainUpdate, TenantBlockData};

#[cfg(feature = "native")]
pub use tracker::Tracker;

// WASM exports
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
EOF

# Fix 2: Create a WASM-compatible time module
echo "📝 Creating src/wasm_time.rs..."
cat > src/wasm_time.rs << 'EOF'
// WASM-compatible time utilities
#[cfg(target_arch = "wasm32")]
pub fn current_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
EOF

# Fix 3: Update blockchain mod.rs to be WASM-compatible
echo "📝 Backing up and fixing src/blockchain/mod.rs..."
cp src/blockchain/mod.rs src/blockchain/mod.rs.backup

# Remove problematic enterprise types from WASM builds
sed -i '/^#\[derive.*TenantBlockchainUpdate/,/^}/c\
// Enterprise types only available in native builds\
#[cfg(feature = "native")]\
#[derive(Debug, Clone, Serialize, Deserialize)]\
pub struct TenantBlockchainUpdate {\
    pub network_id: String,\
    pub peer_id: String,\
    pub new_blocks: Vec<TenantBlockData>,\
    pub timestamp: u64,\
}\
\
#[cfg(feature = "native")]\
#[derive(Debug, Clone, Serialize, Deserialize)]\
pub struct TenantBlockData {\
    pub block_id: u64,\
    pub block_hash: String,\
    pub transactions: Vec<String>,\
    pub timestamp: u64,\
    pub previous_hash: String,\
}' src/blockchain/mod.rs

# Fix 4: Make tenant_blocks field conditional
sed -i 's/tenant_blocks: Vec<TenantBlockData>,/#[cfg(feature = "native")]\n    tenant_blocks: Vec<TenantBlockData>,/' src/blockchain/mod.rs

# Fix 5: Update constructor
sed -i '/tenant_blocks: Vec::new(),/c\
            #[cfg(feature = "native")]\
            tenant_blocks: Vec::new(),' src/blockchain/mod.rs

# Fix 6: Make enterprise methods conditional
sed -i 's/pub fn add_tenant_blocks/#[cfg(feature = "native")]\n    pub fn add_tenant_blocks/' src/blockchain/mod.rs
sed -i 's/pub fn get_recent_tenant_blocks/#[cfg(feature = "native")]\n    pub fn get_recent_tenant_blocks/' src/blockchain/mod.rs
sed -i 's/pub fn get_tenant_summaries/#[cfg(feature = "native")]\n    pub fn get_tenant_summaries/' src/blockchain/mod.rs

echo "✅ WASM fixes applied!"
echo ""
echo "🔨 Now try building WASM:"
echo "   ./build_wasm.sh"
echo ""
echo "🔄 If you need to restore blockchain/mod.rs:"
echo "   cp src/blockchain/mod.rs.backup src/blockchain/mod.rs"

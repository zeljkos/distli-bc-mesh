#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod blockchain;

// Only include tracker and enterprise modules in native builds
#[cfg(feature = "native")]
pub mod tracker;

#[cfg(feature = "native")]
pub mod enterprise_bc;

// Re-export main types from unified blockchain
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

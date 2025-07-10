// js/wasm-loader.js - WASM Module Initialization
import init, { Blockchain, OrderBook } from '../pkg/distli_mesh_bc.js';

let wasmModule;
let blockchain;
let orderBook;

async function initializeWasm() {
    try {
        console.log('🚀 Loading WASM module...');
        wasmModule = await init();
        
        console.log('🧱 Creating Rust blockchain...');
        blockchain = new Blockchain();
        orderBook = new OrderBook();
        
        console.log('✅ WASM blockchain initialized successfully');
        
        // Make globally available
        window.blockchain = blockchain;
        window.orderBook = orderBook;
        
        // Dispatch event to notify other modules
        window.dispatchEvent(new CustomEvent('wasmReady', {
            detail: { blockchain, orderBook }
        }));
        
        return { blockchain, orderBook };
        
    } catch (error) {
        console.error('❌ Failed to initialize WASM:', error);
        throw error;
    }
}

// Export for use in other modules
export { initializeWasm, blockchain, orderBook };

// Main Application Initialization

// Global instances
window.mesh = new MeshManager();
window.blockchain = new SmartBlockchain();

// Initialize application when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    initializeApplication();
});

	function initializeApplication() {
	    log('Initializing Distli Mesh BC...');
	    
	    // Initialize UI components
	    initializeTabs();
	    initializeTradingUI();
	    setupEventListeners();
	    setupModalHandlers();
	    
	    // Initialize contract manager
	    if (typeof SimpleContractManager !== 'undefined') {
		window.contractManager = new SimpleContractManager();
	    }
	    
	    // Set default server
	    const serverInput = document.getElementById('server-input');
	    if (serverInput) {
		serverInput.value = `${window.location.hostname}:3030`;
	    }
	    
	    // Initial UI update
	    updateUI();
	    updatePeersList();
	    
	    // Initialize order book from smart contract
	    setTimeout(() => {
		if (typeof updateOrderBookFromContract === 'function') {
		    updateOrderBookFromContract();
		}
	    }, 200);

	// FIXED: Add cleanup of potentially corrupted state on startup
	setTimeout(() => {
	    if (window.blockchain) {
		// Clear any corrupted cross-network trading data
		const tradingContract = window.blockchain.contract_vm?.contracts?.get("trading_contract");
		if (tradingContract?.state?.trades) {
		    // Remove any trades with network names as traders (corrupted data)
		    const cleanTrades = tradingContract.state.trades.filter(trade => 
			!trade.buyer.includes('_network') && !trade.seller.includes('_network')
		    );
		    if (cleanTrades.length !== tradingContract.state.trades.length) {
			console.log('🧹 Cleaned corrupted cross-network trade data');
			tradingContract.state.trades = cleanTrades;
			window.blockchain.saveToStorage();
		    }
		}
	    }
	}, 2000);
	    
	    log('Multi-language blockchain with smart contracts started');
	    log('Available runtimes: Rust, Python, WASM, JavaScript');
	    log('Contract editor integrated - deploy and test now work with direct blockchain access');
	}

// Export global functions for backwards compatibility
window.placeBuyOrder = placeBuyOrder;
window.placeSellOrder = placeSellOrder;
window.updateOrderBookFromContract = updateOrderBookFromContract;
window.cancelContractOrder = cancelContractOrder;
window.callPythonPredictor = callPythonPredictor;
window.callWasmDeFi = callWasmDeFi;
window.callJavaScriptAnalytics = callJavaScriptAnalytics;
window.testBlockchainSync = testBlockchainSync;
window.showContractState = showContractState;
window.editContract = editContract;
window.saveContract = saveContract;
window.closeContractEditor = closeContractEditor;
window.showNewContractDialog = showNewContractDialog;
window.closeNewContractDialog = closeNewContractDialog;
window.createNewContract = createNewContract;
window.loadContractTemplate = loadContractTemplate;
window.refreshContracts = refreshContracts;
window.deleteContract = deleteContract;
window.toggleExecutionMonitor = toggleExecutionMonitor;
window.clearExecutionLog = clearExecutionLog;
window.updateUI = updateUI;
window.log = log;
window.getUserId = getUserId;

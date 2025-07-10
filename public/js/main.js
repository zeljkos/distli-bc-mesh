// js/main.js - Main Application Initialization and Coordination

import { initializeWasm } from './wasm-loader.js';
import { EnhancedMeshManager } from './mesh-manager.js';
import { tradingManager } from './trading.js';
import { blockchainUI } from './blockchain-ui.js';

class DistliMeshApp {
    constructor() {
        this.mesh = null;
        this.blockchain = null;
        this.orderBook = null;
        this.initialized = false;
        
        // Bind methods
        this.init = this.init.bind(this);
        this.setupEventListeners = this.setupEventListeners.bind(this);
        this.handleError = this.handleError.bind(this);
        this.showTab = this.showTab.bind(this);
    }

    async init() {
        try {
            console.log('Initializing Distli Mesh BC Application...');
            
            // Step 1: Initialize WASM blockchain
            console.log('Loading WASM blockchain module...');
            const { blockchain, orderBook } = await initializeWasm();
            this.blockchain = blockchain;
            this.orderBook = orderBook;
            
            // Step 2: Initialize enhanced mesh manager
            console.log('Setting up mesh networking...');
            this.mesh = new EnhancedMeshManager();
            window.mesh = this.mesh; // Global access
            
            // Notify other modules that mesh is ready
            window.dispatchEvent(new CustomEvent('meshReady', {
                detail: { mesh: this.mesh }
            }));
            
            // Step 3: Set up initial UI state
            console.log('Configuring user interface...');
            this.setupUI();
            
            // Step 4: Set up event listeners
            console.log('Binding event listeners...');
            this.setupEventListeners();
            
            // Step 5: Initialize trading manager
            console.log('Initializing trading system...');
            tradingManager.init();
            
            // Step 6: Auto-connect to default server
            console.log('Setting up network connection...');
            this.setupDefaultConnection();
            
            this.initialized = true;
            console.log('Distli Mesh BC Application initialized successfully');
            
        } catch (error) {
            this.handleError('Failed to initialize application', error);
        }
    }

    setupUI() {
        // Set default server
        const serverInput = document.getElementById('server-input');
        if (serverInput) {
            serverInput.value = `${window.location.hostname}:3030`;
        }
        
        // Initial UI update
        blockchainUI.updateUI();
        
        // Set up tab functionality
        this.setupTabSwitching();
    }

    setupTabSwitching() {
        const tabs = document.querySelectorAll('.tab');
        tabs.forEach(tab => {
            tab.addEventListener('click', (e) => {
                e.preventDefault();
                const tabText = e.target.textContent.trim();
                let targetTab;
                
                if (tabText.includes('Messaging')) targetTab = 'messaging';
                else if (tabText.includes('Trading')) targetTab = 'trading';
                else if (tabText.includes('Order')) targetTab = 'orderbook';
                else if (tabText.includes('Smart')) targetTab = 'contracts';
                else if (tabText.includes('Contract')) targetTab = 'editor';
                else targetTab = tabText.toLowerCase();
                
                if (targetTab) {
                    this.showTab(targetTab);
                }
            });
        });
    }

    showTab(tabName) {
        // Hide all tabs
        document.querySelectorAll('.tab-panel').forEach(panel => {
            panel.classList.remove('active');
        });
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        
        // Show selected tab
        const selectedPanel = document.getElementById(tabName);
        if (selectedPanel) selectedPanel.classList.add('active');
        
        // Find and activate the corresponding tab button
        document.querySelectorAll('.tab').forEach(tab => {
            const tabText = tab.textContent.trim();
            let tabId = '';
            if (tabText.includes('Messaging')) tabId = 'messaging';
            else if (tabText.includes('Trading')) tabId = 'trading';
            else if (tabText.includes('Order')) tabId = 'orderbook';
            else if (tabText.includes('Smart')) tabId = 'contracts';
            else if (tabText.includes('Contract')) tabId = 'editor';
            
            if (tabId === tabName) {
                tab.classList.add('active');
            }
        });
        
        // Update blockchain display when switching tabs
        if (window.blockchainUI && window.blockchainUI.updateBlockchain) {
            window.blockchainUI.updateBlockchain();
        }
    }

    setupEventListeners() {
        // Connection management
        this.setupConnectionListeners();
        
        // Network management
        this.setupNetworkListeners();
        
        // Messaging
        this.setupMessagingListeners();
        
        // Trading
        this.setupTradingListeners();
        
        // Error handling
        this.setupErrorHandling();
        
        // Window events
        this.setupWindowEvents();
    }

    setupConnectionListeners() {
        const connectBtn = document.getElementById('connect-btn');
        if (connectBtn) {
            connectBtn.addEventListener('click', async () => {
                if (this.mesh.connected) {
                    this.disconnectFromTracker();
                } else {
                    await this.connectToTracker();
                }
            });
        }
    }

    setupNetworkListeners() {
        // Join network button
        const joinNetworkBtn = document.getElementById('join-network-btn');
        if (joinNetworkBtn) {
            joinNetworkBtn.addEventListener('click', () => {
                this.joinNetwork();
            });
        }

        // Refresh networks button
        const refreshBtn = document.getElementById('refresh-networks-btn');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => {
                if (this.mesh) {
                    this.mesh.refreshNetworkList();
                }
            });
        }

        // Discover peers button
        const discoverBtn = document.getElementById('discover-btn');
        if (discoverBtn) {
            discoverBtn.addEventListener('click', () => {
                this.discoverPeers();
            });
        }

        // Connect all peers button
        const connectAllBtn = document.getElementById('connect-all-btn');
        if (connectAllBtn) {
            connectAllBtn.addEventListener('click', () => {
                this.connectToAllPeers();
            });
        }
    }

    setupMessagingListeners() {
        // Message input enter key
        const messageInput = document.getElementById('message-input');
        if (messageInput) {
            messageInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    const messageBtn = document.getElementById('message-btn');
                    if (messageBtn && !messageBtn.disabled) {
                        window.sendMessage();
                    }
                }
            });
        }

        // Message button click
        const messageBtn = document.getElementById('message-btn');
        if (messageBtn) {
            messageBtn.addEventListener('click', () => {
                window.sendMessage();
            });
        }
    }

    setupTradingListeners() {
        // Buy order button
        const buyOrderBtn = document.getElementById('buy-order-btn');
        if (buyOrderBtn) {
            buyOrderBtn.addEventListener('click', () => {
                window.placeBuyOrder();
            });
        }

        // Sell order button
        const sellOrderBtn = document.getElementById('sell-order-btn');
        if (sellOrderBtn) {
            sellOrderBtn.addEventListener('click', () => {
                window.placeSellOrder();
            });
        }

        // Order book refresh
        const refreshOrderBookBtn = document.getElementById('refresh-orderbook-btn');
        if (refreshOrderBookBtn) {
            refreshOrderBookBtn.addEventListener('click', () => {
                window.updateOrderBook();
            });
        }

        // Enter key support for trading forms
        this.setupTradingFormListeners();
    }

    setupTradingFormListeners() {
        // Buy form
        const buyInputs = ['buy-quantity', 'buy-price'];
        buyInputs.forEach(inputId => {
            const input = document.getElementById(inputId);
            if (input) {
                input.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') {
                        window.placeBuyOrder();
                    }
                });
            }
        });

        // Sell form
        const sellInputs = ['sell-quantity', 'sell-price'];
        sellInputs.forEach(inputId => {
            const input = document.getElementById(inputId);
            if (input) {
                input.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') {
                        window.placeSellOrder();
                    }
                });
            }
        });
    }

    setupErrorHandling() {
        // Global error handler
        window.addEventListener('error', (event) => {
            console.error('Global error:', event.error);
            this.handleError('Application error', event.error);
        });

        // Unhandled promise rejection handler
        window.addEventListener('unhandledrejection', (event) => {
            console.error('Unhandled promise rejection:', event.reason);
            this.handleError('Promise rejection', event.reason);
        });
    }

    setupWindowEvents() {
        // Page visibility change (for pausing/resuming updates)
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                console.log('Page hidden - reducing update frequency');
            } else {
                console.log('Page visible - resuming normal updates');
                blockchainUI.updateUI();
            }
        });

        // Before unload (cleanup)
        window.addEventListener('beforeunload', () => {
            this.cleanup();
        });
    }

    async connectToTracker() {
        try {
            console.log('Connecting to tracker...');
            await this.mesh.connect();
        } catch (error) {
            this.handleError('Failed to connect to tracker', error);
        }
    }

    disconnectFromTracker() {
        try {
            console.log('Disconnecting from tracker...');
            if (this.mesh.ws) {
                this.mesh.ws.close();
            }
        } catch (error) {
            this.handleError('Failed to disconnect from tracker', error);
        }
    }

    joinNetwork() {
        const networkId = this.mesh.getSelectedNetwork();
        if (networkId) {
            console.log(`Joining network: ${networkId}`);
            this.mesh.joinNetwork(networkId);
        } else {
            this.showAlert('Please select or enter a network name', 'warning');
        }
    }

    discoverPeers() {
        if (this.mesh) {
            console.log('Discovering peers in network...');
            this.mesh.send({ type: 'peers', peers: [] });
        }
    }

    async connectToAllPeers() {
        if (this.mesh && this.mesh.availablePeers.length > 0) {
            console.log(`Connecting to ${this.mesh.availablePeers.length} peers...`);
            await this.mesh.connectToAllPeers();
        } else {
            this.showAlert('No peers available to connect to', 'info');
        }
    }

    setupDefaultConnection() {
        // Set up server input default
        const serverInput = document.getElementById('server-input');
        if (serverInput && !serverInput.value.trim()) {
            serverInput.value = `${window.location.hostname}:3030`;
        }
    }

    showAlert(message, type = 'info') {
        blockchainUI.showAlert(message, type);
    }

    handleError(context, error) {
        console.error(`${context}:`, error);
        
        const errorMsg = error?.message || error?.toString() || 'Unknown error';
        this.showAlert(`${context}: ${errorMsg}`, 'error');
        
        // For critical errors, show more detailed feedback
        if (context.includes('initialize') || context.includes('WASM')) {
            const criticalError = document.createElement('div');
            criticalError.style.cssText = `
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                background: #dc3545;
                color: white;
                padding: 15px;
                text-align: center;
                z-index: 3000;
                font-weight: bold;
            `;
            criticalError.innerHTML = `
                Critical Error: ${context} - Please refresh the page
                <button onclick="window.location.reload()" 
                        style="margin-left: 15px; background: white; color: #dc3545; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer;">
                    Refresh
                </button>
            `;
            document.body.appendChild(criticalError);
        }
    }

    // Utility methods
    getAppState() {
        return {
            initialized: this.initialized,
            connected: this.mesh?.connected || false,
            currentNetwork: this.mesh?.currentNetwork || null,
            peerCount: this.mesh?.dataChannels.size || 0,
            blockCount: this.blockchain?.get_chain_length() || 0,
            pendingTransactions: this.blockchain?.get_pending_count() || 0
        };
    }

    // Cleanup method
    cleanup() {
        console.log('Cleaning up application...');
        
        try {
            // Disconnect from tracker
            if (this.mesh?.ws) {
                this.mesh.ws.close();
            }
            
            // Close P2P connections
            if (this.mesh?.dataChannels) {
                this.mesh.dataChannels.forEach(channel => {
                    if (channel.readyState === 'open') {
                        channel.close();
                    }
                });
            }
            
            // Stop UI updates
            if (blockchainUI?.stopUpdateLoop) {
                blockchainUI.stopUpdateLoop();
            }
            
        } catch (error) {
            console.error('Error during cleanup:', error);
        }
    }

    // Debug method
    debug() {
        return {
            app: this.getAppState(),
            mesh: {
                connected: this.mesh?.connected,
                offlineMode: this.mesh?.offlineMode,
                pendingMessages: this.mesh?.pendingMessages.length,
                dataChannels: this.mesh?.dataChannels.size,
                peers: this.mesh?.peers.size
            },
            blockchain: {
                height: this.blockchain?.get_chain_length(),
                pending: this.blockchain?.get_pending_count(),
                validators: this.blockchain?.get_validator_count()
            }
        };
    }
}

// Initialize application when DOM is loaded
document.addEventListener('DOMContentLoaded', async () => {
    try {
        // Create global app instance
        window.distliApp = new DistliMeshApp();
        
        // Initialize the application
        await window.distliApp.init();
        
        // Make functions available globally
        window.debug = () => window.distliApp.debug();
        window.showTab = (tabName) => window.distliApp.showTab(tabName);
        
        console.log('Application ready! Type debug() in console for diagnostics.');
        
    } catch (error) {
        console.error('Failed to start application:', error);
        alert('Failed to start Distli Mesh BC. Please refresh the page.');
    }
});

// Handle hot reloading in development
if (import.meta.hot) {
    import.meta.hot.accept(() => {
        console.log('Hot reloading...');
        window.location.reload();
    });
}

export { DistliMeshApp };

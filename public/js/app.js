// Enhanced app.js - Integrated Leadership System
// File: public/js/app.js

// Global instances
window.mesh = new MeshManager();
window.blockchain = new SmartBlockchain();
window.leadershipManager = null; // Will be initialized after network connection

// Initialize application when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    initializeApplication();
});

function initializeApplication() {
    log('Initializing Distli Mesh BC with Robust Leadership...');
    
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
    
    // Enhanced UI with leadership indicators
    addLeadershipUI();
    
    // Initial UI update
    updateUI();
    updatePeersList();
    
    // Initialize order book from smart contract
    setTimeout(() => {
        if (typeof updateOrderBookFromContract === 'function') {
            updateOrderBookFromContract();
        }
    }, 200);

    // Cleanup corrupted state on startup
    setTimeout(() => {
        if (window.blockchain) {
            const tradingContract = window.blockchain.contract_vm?.contracts?.get("trading_contract");
            if (tradingContract?.state?.trades) {
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
    
    log('Multi-language blockchain with robust leadership started');
    log('Available runtimes: Rust, Python, WASM, JavaScript');
    log('Distributed leadership system ready for network formation');
}

// NEW: Add leadership UI elements
function addLeadershipUI() {
    const statusContainer = document.querySelector('.status');
    if (statusContainer) {
        // Add leadership status indicator
        const leadershipDiv = document.createElement('div');
        leadershipDiv.className = 'stat';
        leadershipDiv.innerHTML = `
            <strong id="leadership-status">Initializing</strong>
            <div>Leadership</div>
        `;
        statusContainer.appendChild(leadershipDiv);
        
        // Add leadership debug panel
        const container = document.querySelector('.container');
        if (container) {
            const debugPanel = document.createElement('div');
            debugPanel.className = 'container';
            debugPanel.style.display = 'none';
            debugPanel.id = 'leadership-debug-panel';
            debugPanel.innerHTML = `
                <h3>🏛️ Leadership System Debug</h3>
                <div style="margin-bottom: 15px;">
                    <button onclick="toggleLeadershipDebug()" class="btn btn-primary">Toggle Debug</button>
                    <button onclick="forceLeadershipElection()" class="btn btn-warning">Force Election</button>
                    <button onclick="showLeadershipStatus()" class="btn btn-info">Show Status</button>
                </div>
                <div id="leadership-debug-content" class="debug-log"></div>
            `;
            container.parentNode.insertBefore(debugPanel, container.nextSibling);
        }
    }
}

// NEW: Initialize robust leadership when network is joined
function initializeRobustLeadership() {
    if (window.leadershipManager) {
        console.log('🔄 Destroying existing leadership manager');
        window.leadershipManager.destroy();
    }
    
    if (window.mesh && window.blockchain && window.mesh.currentNetwork) {
        console.log('🏗️ Initializing robust leadership system');
        window.leadershipManager = new DistributedLeadershipManager(
            window.blockchain, 
            window.mesh
        );
        
        // Update UI when leadership changes
        window.leadershipManager.onLeadershipChange = (isLeader, leaderId) => {
            updateLeadershipUI(isLeader, leaderId);
            updateUI(); // Refresh main UI
        };
        
        log('Robust leadership system initialized for network: ' + window.mesh.currentNetwork);
    } else {
        console.warn('Cannot initialize leadership - missing dependencies');
    }
}

// NEW: Update leadership UI indicators
function updateLeadershipUI(isLeader, leaderId) {
    const statusEl = document.getElementById('leadership-status');
    const networkNameEl = document.getElementById('network-name');
    
    if (statusEl) {
        if (isLeader) {
            statusEl.textContent = 'Leader 👑';
            statusEl.style.color = '#28a745';
            statusEl.style.fontWeight = 'bold';
        } else if (leaderId) {
            statusEl.textContent = `Follower`;
            statusEl.style.color = '#6c757d';
            statusEl.style.fontWeight = 'normal';
        } else {
            statusEl.textContent = 'Election 🗳️';
            statusEl.style.color = '#ffc107';
            statusEl.style.fontWeight = 'normal';
        }
    }
    
    // Update mining button based on leadership
    updateMiningButton(isLeader);
    
    // Update peer list with leadership indicators
    updatePeersList();
    
    // Log leadership change
    if (isLeader) {
        log('👑 Became network leader - mining enabled');
    } else if (leaderId) {
        log(`👥 Following leader: ${leaderId.substring(0, 8)}`);
    } else {
        log('🗳️ Leadership election in progress');
    }
}

// NEW: Enhanced mining button updates
function updateMiningButton(isLeader) {
    const mineBtn = document.getElementById('mine-btn');
    if (!mineBtn || !window.blockchain) return;
    
    const hasPending = window.blockchain.pending.length > 0;
    const hasNetwork = window.mesh && window.mesh.currentNetwork;
    
    if (isLeader && hasPending && hasNetwork) {
        mineBtn.disabled = false;
        mineBtn.textContent = `⛏️ Mine Block (Leader) - ${window.blockchain.pending.length} pending`;
        mineBtn.style.background = '#28a745';
        mineBtn.style.boxShadow = '0 0 10px rgba(40, 167, 69, 0.3)';
        mineBtn.classList.add('leader-mine-btn');
    } else if (!isLeader && hasNetwork) {
        mineBtn.disabled = true;
        mineBtn.textContent = '⏳ Leader Will Mine - Not Mining Leader';
        mineBtn.style.background = '#6c757d';
        mineBtn.style.boxShadow = 'none';
        mineBtn.classList.remove('leader-mine-btn');
        mineBtn.classList.add('follower-mine-btn');
    } else if (!hasNetwork) {
        mineBtn.disabled = true;
        mineBtn.textContent = 'Connect to Network First';
        mineBtn.style.background = '#dc3545';
        mineBtn.style.boxShadow = 'none';
    } else {
        mineBtn.disabled = true;
        mineBtn.textContent = 'No Pending Transactions';
        mineBtn.style.background = '#6c757d';
        mineBtn.style.boxShadow = 'none';
    }
}

// Enhanced network joining with leadership initialization
async function joinNetwork() {
    const networkId = window.mesh.getSelectedNetwork();
    if (!networkId) {
        alert('Please select or enter a network name');
        return;
    }
    
    console.log('🌐 Joining network:', networkId);
    window.mesh.joinNetwork(networkId);
    
    // Wait for network connection and then initialize leadership
    setTimeout(() => {
        if (window.mesh.currentNetwork === networkId) {
            initializeRobustLeadership();
        }
    }, 1000);
}

// Enhanced mining with leadership verification
function mineBlock() {
    if (!window.blockchain || !window.leadershipManager) {
        alert('Blockchain or leadership system not initialized');
        return;
    }
    
    if (!window.leadershipManager.isLeader()) {
        alert('You are not the verified leader. Mining is reserved for the network leader.');
        log('❌ Mining rejected - not the verified leader');
        return;
    }
    
    if (window.blockchain.pending.length === 0) {
        alert('No pending transactions to mine');
        return;
    }
    
    console.log('⛏️ Mining as verified leader...');
    const block = window.blockchain.mineBlock();
    
    if (block) {
        log(`✅ Block #${block.id} mined by verified leader`);
        
        // Broadcast the block
        if (window.mesh.broadcast) {
            window.mesh.broadcast({ type: 'block', block: block });
        }
        
        // Update UI
        updateUI();
        if (typeof updateOrderBookFromContract === 'function') {
            updateOrderBookFromContract();
        }
        
        console.log(`📡 Block #${block.id} broadcasted to network`);
    } else {
        log('❌ Mining failed');
    }
}

// Enhanced UI update with leadership integration
function updateUI() {
    if (!window.mesh) return;
    
    const peerCountEl = document.getElementById('peer-count');
    const blockCountEl = document.getElementById('block-count');
    const pendingCountEl = document.getElementById('pending-count');
    const statusEl = document.getElementById('status');
    const networkNameEl = document.getElementById('network-name');
    
    // Basic stats
    if (peerCountEl) peerCountEl.textContent = window.mesh.dataChannels ? window.mesh.dataChannels.size : 0;
    if (blockCountEl && window.blockchain) blockCountEl.textContent = window.blockchain.chain.length;
    if (pendingCountEl && window.blockchain) pendingCountEl.textContent = window.blockchain.pending.length;
    if (networkNameEl) networkNameEl.textContent = window.mesh.currentNetwork || 'None';
    
    // Enhanced status with leadership
    if (statusEl) {
        let statusText = window.mesh.connected ? 'Connected' : 'Offline';
        
        if (window.mesh.connected && window.leadershipManager) {
            const leadershipState = window.leadershipManager.getState();
            switch (leadershipState.state) {
                case 'LEADER':
                    statusText += ' (Verified Leader 👑)';
                    break;
                case 'CANDIDATE':
                    statusText += ' (Election 🗳️)';
                    break;
                case 'FOLLOWER':
                    if (leadershipState.currentLeader) {
                        statusText += ` (Follower 👥)`;
                    } else {
                        statusText += ' (No Leader 🔍)';
                    }
                    break;
            }
        }
        
        statusEl.textContent = statusText;
    }
    
    // Update mining button
    if (window.leadershipManager) {
        updateMiningButton(window.leadershipManager.isLeader());
    }
    
    // Update other UI elements
    updateButtons();
    updateBlockchain();
    updatePeersList();
}

// Enhanced buttons update with leadership awareness
function updateButtons() {
    const hasNetwork = window.mesh && window.mesh.currentNetwork;
    const isConnected = window.mesh && window.mesh.connected;
    const isLeader = window.leadershipManager ? window.leadershipManager.isLeader() : false;
    const hasPending = window.blockchain && window.blockchain.pending.length > 0;
    
    // Connection button
    const connectBtn = document.getElementById('connect-btn');
    if (connectBtn) {
        connectBtn.textContent = isConnected ? 'Disconnect' : 'Connect';
    }
    
    // Network buttons
    const joinNetworkBtn = document.getElementById('join-network-btn');
    const discoverBtn = document.getElementById('discover-btn');
    const connectAllBtn = document.getElementById('connect-all-btn');
    const sendBtn = document.getElementById('send-btn');
    const syncBtn = document.getElementById('sync-blockchain-btn');
    
    if (joinNetworkBtn) joinNetworkBtn.disabled = !isConnected;
    if (discoverBtn) discoverBtn.disabled = !hasNetwork;
    if (connectAllBtn) {
        connectAllBtn.disabled = !hasNetwork || !window.mesh.availablePeers || window.mesh.availablePeers.length === 0;
    }
    if (sendBtn) sendBtn.disabled = !hasNetwork;
    if (syncBtn) syncBtn.disabled = !hasNetwork || !window.blockchain || window.blockchain.chain.length <= 1;
}

// Debug functions for leadership
function toggleLeadershipDebug() {
    const panel = document.getElementById('leadership-debug-panel');
    if (panel) {
        panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
    }
}

function showLeadershipStatus() {
    if (!window.leadershipManager) {
        alert('Leadership manager not initialized');
        return;
    }
    
    const debugInfo = window.leadershipManager.getDebugInfo();
    const content = document.getElementById('leadership-debug-content');
    
    if (content) {
        content.innerHTML = `
            <div><strong>Leadership Debug Info:</strong></div>
            <div>Node ID: ${debugInfo.nodeId}</div>
            <div>State: ${debugInfo.state}</div>
            <div>Current Leader: ${debugInfo.currentLeader || 'None'}</div>
            <div>Term: ${debugInfo.term}</div>
            <div>Total Peers: ${debugInfo.totalPeers}</div>
            <div>Network: ${debugInfo.networkId || 'None'}</div>
            <div>Pending Transactions: ${debugInfo.pendingTransactions}</div>
            <div><strong>Last Heartbeats:</strong></div>
            ${Object.entries(debugInfo.lastHeartbeats).map(([id, time]) => 
                `<div>  ${id}: ${time}</div>`
            ).join('')}
        `;
    }
    
    console.table(debugInfo);
    log('Leadership status displayed in debug panel');
}

function forceLeadershipElection() {
    if (!window.leadershipManager) {
        alert('Leadership manager not initialized');
        return;
    }
    
    if (confirm('Force a new leadership election? This will trigger network-wide leadership selection.')) {
        window.leadershipManager.forceElection();
        log('🗳️ Forced leadership election triggered');
    }
}

// Enhanced event listeners with leadership integration
function setupEventListeners() {
    const connectBtn = document.getElementById('connect-btn');
    if (connectBtn) {
        connectBtn.addEventListener('click', async () => {
            if (window.mesh.connected) {
                // Cleanup leadership when disconnecting
                if (window.leadershipManager) {
                    window.leadershipManager.destroy();
                    window.leadershipManager = null;
                }
                window.mesh.ws.close();
            } else {
                await window.mesh.connect();
            }
        });
    }

    const joinNetworkBtn = document.getElementById('join-network-btn');
    if (joinNetworkBtn) {
        joinNetworkBtn.addEventListener('click', joinNetwork);
    }

    const refreshNetworksBtn = document.getElementById('refresh-networks-btn');
    if (refreshNetworksBtn) {
        refreshNetworksBtn.addEventListener('click', () => {
            window.mesh.refreshNetworkList();
        });
    }

    const discoverBtn = document.getElementById('discover-btn');
    if (discoverBtn) {
        discoverBtn.addEventListener('click', () => {
            window.mesh.send({ type: 'peers', peers: [] });
            log('Discovering peers in network...');
        });
    }

    const connectAllBtn = document.getElementById('connect-all-btn');
    if (connectAllBtn) {
        connectAllBtn.addEventListener('click', () => {
            window.mesh.connectToAll();
            log('Connecting to all peers in network...');
        });
    }

    const sendBtn = document.getElementById('send-btn');
    if (sendBtn) {
        sendBtn.addEventListener('click', () => {
            const input = document.getElementById('tx-input');
            if (!input || !window.blockchain) return;
            
            const data = input.value.trim();
            if (data) {
                const tx = window.blockchain.addMessage(data, getUserId());
                
                // Broadcast transaction to network
                const txMessage = { type: 'transaction', transaction: tx };
                if (window.mesh.connected) window.mesh.send(txMessage);
                window.mesh.broadcast(txMessage);
                
                log(`Sent message: ${data}`);
                input.value = '';
                updateUI();
            }
        });
    }

    // Enhanced mine button with leadership verification
    const mineBtn = document.getElementById('mine-btn');
    if (mineBtn) {
        mineBtn.addEventListener('click', mineBlock);
    }

    const syncBtn = document.getElementById('sync-blockchain-btn');
    if (syncBtn) {
        syncBtn.addEventListener('click', testBlockchainSync);
    }
}

// Export enhanced global functions
window.initializeRobustLeadership = initializeRobustLeadership;
window.updateLeadershipUI = updateLeadershipUI;
window.updateMiningButton = updateMiningButton;
window.joinNetwork = joinNetwork;
window.mineBlock = mineBlock;
window.toggleLeadershipDebug = toggleLeadershipDebug;
window.showLeadershipStatus = showLeadershipStatus;
window.forceLeadershipElection = forceLeadershipElection;

// Existing exports
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

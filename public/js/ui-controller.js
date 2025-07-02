// UI Controller and Event Handlers - FIXED VERSION with Leader Status

// Tab switching functionality
function initializeTabs() {
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            tab.classList.add('active');
            const targetTab = document.getElementById(tab.dataset.tab + '-tab');
            if (targetTab) {
                targetTab.classList.add('active');
            }
        });
    });
}

// ENHANCED: UI update function with leader status and better coordination info
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
    
    // ENHANCED: Status with leader information
    if (statusEl) {
        let statusText = window.mesh.connected ? 'Connected' : 'Offline';
        
        if (window.blockchain && window.mesh.connected) {
            if (window.blockchain.isMiningLeader) {
                statusText += ' (Mining Leader 👑)';
            } else {
                statusText += ' (Follower 👥)';
            }
        }
        
        statusEl.textContent = statusText;
        
        // Add CSS classes for visual indication
        statusEl.className = '';
        if (window.mesh.connected) {
            statusEl.classList.add('connected');
            if (window.blockchain && window.blockchain.isMiningLeader) {
                statusEl.classList.add('leader');
            }
        }
    }

    // Calculate states for button enabling/disabling
    const hasNetwork = window.mesh.currentNetwork;
    const hasPeers = window.mesh.dataChannels && window.mesh.dataChannels.size > 0;
    const hasBlocks = window.blockchain && window.blockchain.chain.length > 1;
    const isLeader = window.blockchain && window.blockchain.isMiningLeader;
    const hasPending = window.blockchain && window.blockchain.pending.length > 0;
    
    // Update buttons with enhanced logic
    updateButtons(hasNetwork, hasPeers, hasBlocks, isLeader, hasPending);
    
    updateBlockchain();
    updatePeersList();
}

// NEW: Enhanced button update logic
	function updateButtons(hasNetwork, hasPeers, hasBlocks, isLeader, hasPending) {
	    const connectBtn = document.getElementById('connect-btn');
	    const joinNetworkBtn = document.getElementById('join-network-btn');
	    const discoverBtn = document.getElementById('discover-btn');
	    const connectAllBtn = document.getElementById('connect-all-btn');
	    const sendBtn = document.getElementById('send-btn');
	    const mineBtn = document.getElementById('mine-btn');
	    const syncBtn = document.getElementById('sync-blockchain-btn');
	    
	    // Connection button
	    if (connectBtn) {
		connectBtn.textContent = window.mesh.connected ? 'Disconnect' : 'Connect';
	    }
	    
	    // Network buttons
	    if (joinNetworkBtn) joinNetworkBtn.disabled = !window.mesh.connected;
	    if (discoverBtn) discoverBtn.disabled = !hasNetwork;
	    if (connectAllBtn) {
		connectAllBtn.disabled = !hasNetwork || !window.mesh.availablePeers || window.mesh.availablePeers.length === 0;
	    }
	    
	    // Communication buttons
	    if (sendBtn) sendBtn.disabled = !hasNetwork;
	    
	    // FIXED: More permissive mining button logic
	    if (mineBtn && window.blockchain) {
		// Allow mining if have pending transactions and (are leader OR no peers)
		const canMine = hasPending && (isLeader || hasPeers === 0);
		mineBtn.disabled = !canMine;
		
		if (isLeader) {
		    mineBtn.textContent = hasPending ? `Mine Block (Leader) - ${window.blockchain.pending.length} pending` : 'Mine Block (Leader)';
		    mineBtn.style.background = hasPending ? '#28a745' : '#6c757d';
		} else if (hasPeers === 0) {
		    mineBtn.textContent = hasPending ? `Mine Block (Solo) - ${window.blockchain.pending.length} pending` : 'Mine Block (Solo)';
		    mineBtn.style.background = hasPending ? '#28a745' : '#6c757d';
		} else {
		    mineBtn.textContent = 'Mine Block (Follower)';
		    mineBtn.style.background = '#dc3545';
		}
	    }
	    
	    // Sync button
	    if (syncBtn) syncBtn.disabled = !hasNetwork || !hasBlocks;
	}
// ENHANCED: Blockchain display with leader and mining information
function updateBlockchain() {
    if (!window.blockchain) return;
    
    // Update pending transactions display with leader context
    const pendingDiv = document.getElementById('pending-transactions');
    if (pendingDiv && window.blockchain.pending.length > 0) {
        const isLeader = window.blockchain.isMiningLeader;
        const pendingCount = window.blockchain.pending.length;
        
        const pendingHtml = window.blockchain.pending.map((tx, i) => {
            if (tx.type === 'contract_call') {
                return `${i+1}. Smart Contract: ${tx.call.function}(${JSON.stringify(tx.call.params)}) - Gas: ${tx.result?.gas_used || 'pending'}`;
            } else if (tx.type === 'message') {
                return `${i+1}. Message: "${tx.data}"`;
            }
            return `${i+1}. ${tx.type}`;
        }).join('<br>');
        
        // Enhanced pending display with leadership context
        let actionButton = '';
        if (isLeader) {
            actionButton = `
                <button onclick="window.blockchain.mineBlock()" 
                        style="background: #28a745; color: white; border: none; padding: 8px 15px; border-radius: 4px; margin-left: 10px;">
                    ⛏️ Mine Block Now (Leader)
                </button>
            `;
        } else {
            actionButton = `
                <span style="color: #6c757d; font-style: italic; margin-left: 10px;">
                    ⏳ Waiting for mining leader to process
                </span>
            `;
        }
        
        pendingDiv.innerHTML = `
            <div style="background: ${isLeader ? '#d4edda' : '#fff3cd'}; padding: 15px; border-radius: 8px; border-left: 4px solid ${isLeader ? '#28a745' : '#ffc107'};">
                <strong>Pending Transactions (${pendingCount}) - ${isLeader ? 'You are Mining Leader 👑' : 'Mining Follower 👥'}:</strong><br>
                ${pendingHtml}
                <br><br>
                ${actionButton}
            </div>
        `;
    } else if (pendingDiv) {
        pendingDiv.innerHTML = '';
    }
    
    // Update blockchain display with mining information
    const blockchainDiv = document.getElementById('blockchain');
    if (blockchainDiv) {
        blockchainDiv.innerHTML = '';
        
        // Show recent blocks (last 10)
        const recentBlocks = window.blockchain.chain.slice(-10);
        
        recentBlocks.forEach((block) => {
            const blockDiv = document.createElement('div');
            blockDiv.className = 'block';
            
            // Enhanced block display with miner information
            const isMiningLeader = window.blockchain.isMiningLeader;
            const minedByMe = block.miner === getUserId();
            
            if (minedByMe) {
                blockDiv.style.borderLeft = '4px solid #28a745';
                blockDiv.style.background = '#d4edda';
            }
            
            let transactionDetails = '';
            if (block.transactions && block.transactions.length > 0) {
                transactionDetails = '<div class="block-data"><strong>Transactions:</strong></div>';
                block.transactions.forEach((tx, i) => {
                    if (tx.type === 'contract_call') {
                        const success = tx.result && tx.result.success ? '✅' : '❌';
                        const gasUsed = tx.result ? tx.result.gas_used : 'N/A';
                        transactionDetails += `
                            <div class="block-data" style="margin-left: 10px; font-size: 12px;">
                                ${i+1}. ${success} Contract: ${tx.call.function}(${JSON.stringify(tx.call.params)}) 
                                [Gas: ${gasUsed}] by ${tx.sender.substring(0, 8)}
                            </div>
                        `;
                    } else if (tx.type === 'message') {
                        transactionDetails += `
                            <div class="block-data" style="margin-left: 10px; font-size: 12px;">
                                ${i+1}. 💬 Message: "${tx.data}" by ${tx.sender.substring(0, 8)}
                            </div>
                        `;
                    } else if (tx.type === 'cross_network_trade') {
                        transactionDetails += `
                            <div class="block-data" style="margin-left: 10px; font-size: 12px;">
                                ${i+1}. 🔄 Cross-trade: ${tx.asset} ${tx.quantity}@${tx.price}
                            </div>
                        `;
                    }
                });
            }
            
            blockDiv.innerHTML = `
                <div class="block-header">
                    Block #${block.id} ${minedByMe ? '👑 (Mined by You)' : ''}
                </div>
                <div class="block-data">Summary: ${block.data}</div>
                <div class="block-data">Hash: ${block.hash.substring(0, 20)}...</div>
                <div class="block-data">Time: ${new Date(block.timestamp).toLocaleTimeString()}</div>
                <div class="block-data">Miner: ${block.miner ? block.miner.substring(0, 8) : 'Unknown'}</div>
                ${transactionDetails}
            `;
            blockchainDiv.appendChild(blockDiv);
        });
    }
}

// ENHANCED: Peers list with leader indication
function updatePeersList() {
    const peersDiv = document.getElementById('peers-list');
    if (!peersDiv || !window.mesh) return;
    
    peersDiv.innerHTML = '';
    
    if (!window.mesh.dataChannels || window.mesh.dataChannels.size === 0) {
        peersDiv.innerHTML = '<div>No peers connected</div>';
        return;
    }
    
    // Show ourselves first if we're the leader
    if (window.blockchain && window.blockchain.isMiningLeader) {
        const selfDiv = document.createElement('div');
        selfDiv.className = 'peer leader-peer';
        selfDiv.style.background = '#d4edda';
        selfDiv.style.border = '2px solid #28a745';
        selfDiv.textContent = `${getUserId().substring(0, 8)} (You - Leader 👑)`;
        peersDiv.appendChild(selfDiv);
    }
    
    // Show other peers
    window.mesh.dataChannels.forEach((dc, peerId) => {
        const peerDiv = document.createElement('div');
        peerDiv.className = 'peer';
        
        // Check if this peer announced themselves as a leader (shouldn't happen with fixes)
        if (window.mesh.knownMiningLeaders && 
            window.mesh.knownMiningLeaders.get(window.mesh.currentNetwork) === peerId) {
            peerDiv.style.background = '#f8d7da';
            peerDiv.style.border = '1px solid #dc3545';
            peerDiv.textContent = `${peerId.substring(0, 8)} ⚠️ (Claims Leader)`;
        } else {
            peerDiv.textContent = peerId.substring(0, 8);
        }
        
        peersDiv.appendChild(peerDiv);
    });

    // Add network status info
    const statusDiv = document.createElement('div');
    statusDiv.style.fontSize = '12px';
    statusDiv.style.color = '#6c757d';
    statusDiv.style.marginTop = '10px';
    statusDiv.innerHTML = `
        Network: ${window.mesh.currentNetwork || 'None'}<br>
        Total Peers: ${window.mesh.dataChannels.size + 1} (including you)<br>
        Leader: ${window.blockchain && window.blockchain.isMiningLeader ? 'You' : 'Other peer'}
    `;
    peersDiv.appendChild(statusDiv);
}

// Event listener setup
function setupEventListeners() {
    const connectBtn = document.getElementById('connect-btn');
    if (connectBtn) {
        connectBtn.addEventListener('click', async () => {
            if (window.mesh.connected) {
                window.mesh.ws.close();
            } else {
                await window.mesh.connect();
            }
        });
    }

    const joinNetworkBtn = document.getElementById('join-network-btn');
    if (joinNetworkBtn) {
        joinNetworkBtn.addEventListener('click', () => {
            const networkId = window.mesh.getSelectedNetwork();
            if (networkId) {
                window.mesh.joinNetwork(networkId);
            } else {
                log('Please select or enter a network name');
            }
        });
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

    // ENHANCED: Mine button with leader checks
    const mineBtn = document.getElementById('mine-btn');
    if (mineBtn) {
        mineBtn.addEventListener('click', () => {
            if (!window.blockchain) return;
            
            // Check if we're the leader
            if (!window.blockchain.isMiningLeader) {
                alert('You are not the mining leader for this network. Another peer will mine pending transactions.');
                log('Mining rejected - not the designated leader');
                return;
            }
            
            if (window.blockchain.pending.length === 0) {
                alert('No pending transactions to mine.');
                return;
            }
            
            const block = window.blockchain.mineBlock();
            if (block) {
                log(`Mined block #${block.id} as leader`);
                updateUI();
                if (typeof updateOrderBookFromContract === 'function') {
                    updateOrderBookFromContract();
                }
                
                console.log(`✅ Block #${block.id} mined and broadcasted to network`);
            }
        });
    }

    const syncBtn = document.getElementById('sync-blockchain-btn');
    if (syncBtn) {
        syncBtn.addEventListener('click', testBlockchainSync);
    }
}

// Modal event handlers
function setupModalHandlers() {
    window.onclick = function(event) {
        const editorModal = document.getElementById('contract-editor-modal');
        const newModal = document.getElementById('new-contract-modal');
        
        if (event.target === editorModal && editorModal) {
            closeContractEditor();
        }
        if (event.target === newModal && newModal) {
            closeNewContractDialog();
        }
    };
}

// NEW: Enhanced monitoring and debugging
function showNetworkStatus() {
    if (!window.blockchain || !window.mesh) {
        console.log('❌ Blockchain or mesh not initialized');
        return;
    }
    
    const status = {
        network: window.mesh.currentNetwork,
        connected: window.mesh.connected,
        peers: window.mesh.dataChannels ? window.mesh.dataChannels.size : 0,
        is_leader: window.blockchain.isMiningLeader,
        pending: window.blockchain.pending.length,
        blocks: window.blockchain.chain.length,
        last_block_miner: window.blockchain.chain.length > 1 ? 
            window.blockchain.chain[window.blockchain.chain.length - 1].miner : 'none'
    };
    
    console.table(status);
    
    // Show in UI
    const statusHtml = Object.entries(status)
        .map(([key, value]) => `<strong>${key}:</strong> ${value}`)
        .join('<br>');
    
    const logDiv = document.getElementById('log');
    if (logDiv) {
        logDiv.innerHTML += `<div style="background: #e3f2fd; padding: 10px; margin: 5px 0; border-radius: 4px;">
            📊 <strong>Network Status:</strong><br>${statusHtml}
        </div>`;
        logDiv.scrollTop = logDiv.scrollHeight;
    }
    
    return status;
}

// NEW: Force leadership (for debugging)
function forceLeadership() {
    if (!window.blockchain) {
        console.log('❌ No blockchain instance');
        return;
    }
    
    console.log('🔧 Forcing leadership status (DEBUG ONLY)');
    window.blockchain.isMiningLeader = true;
    window.blockchain.becomeLeader(window.mesh ? window.mesh.currentNetwork : 'debug');
    updateUI();
    log('Forced mining leadership - USE FOR DEBUGGING ONLY');
}

// NEW: Add CSS for leader styling
function addLeaderStyles() {
    const style = document.createElement('style');
    style.textContent = `
        .leader-mine-btn {
            background: #28a745 !important;
            color: white !important;
            border: 2px solid #1e7e34 !important;
            box-shadow: 0 0 10px rgba(40, 167, 69, 0.3) !important;
        }
        
        .follower-mine-btn {
            background: #dc3545 !important;
            color: white !important;
            opacity: 0.6;
        }
        
        .leader-peer {
            border: 2px solid #28a745 !important;
            background: #d4edda !important;
        }
        
        .connected.leader {
            color: #28a745 !important;
            font-weight: bold !important;
        }
        
        @keyframes pulse {
            0% { box-shadow: 0 0 0 0 rgba(40, 167, 69, 0.7); }
            70% { box-shadow: 0 0 0 10px rgba(40, 167, 69, 0); }
            100% { box-shadow: 0 0 0 0 rgba(40, 167, 69, 0); }
        }
    `;
    document.head.appendChild(style);
}

// Initialize styles when loaded
document.addEventListener('DOMContentLoaded', addLeaderStyles);

// Export functions for global access
window.showNetworkStatus = showNetworkStatus;
window.forceLeadership = forceLeadership;

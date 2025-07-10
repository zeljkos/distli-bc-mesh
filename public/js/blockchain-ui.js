// js/blockchain-ui.js - Clean UI Management without emojis

class BlockchainUI {
    constructor() {
        this.mesh = null;
        this.blockchain = null;
        this.updateInterval = null;
        
        // Bind methods to maintain context
        this.updateUI = this.updateUI.bind(this);
        this.updateBlockchain = this.updateBlockchain.bind(this);
        this.sendMessage = this.sendMessage.bind(this);
        this.mineBlock = this.mineBlock.bind(this);
        
        // Listen for updates from other modules
        window.addEventListener('wasmReady', (event) => {
            this.blockchain = event.detail.blockchain;
            console.log('UI connected to blockchain');
            this.startUpdateLoop();
        });
        
        window.addEventListener('meshReady', (event) => {
            this.mesh = event.detail.mesh;
            console.log('UI connected to mesh');
        });
        
        window.addEventListener('meshUpdate', () => {
            this.updateUI();
        });
        
        window.addEventListener('tradingUpdate', () => {
            this.updateUI();
        });
    }

    updateUI() {
        if (!this.blockchain) return;
        
        try {
            const connectionStatus = this.mesh ? this.mesh.getConnectionStatus() : { status: 'offline', connections: 0 };
            
            // Update basic stats
            this.updateConnectionStats(connectionStatus);
            
            // Update button states
            this.updateButtonStates(connectionStatus);
            
            // Update validators if needed
            this.ensureValidators(connectionStatus);
            
            // Update blockchain display in all tabs
            this.updateBlockchain();
            
        } catch (error) {
            console.error('Error updating UI:', error);
        }
    }

    updateConnectionStats(connectionStatus) {
        // Update peer count
        const peerCountEl = document.getElementById('peer-count');
        if (peerCountEl) {
            peerCountEl.textContent = connectionStatus.connections;
        }
        
        // Update block count
        const blockCountEl = document.getElementById('block-count');
        if (blockCountEl) {
            blockCountEl.textContent = this.blockchain.get_chain_length();
        }
        
        // Update network name
        const networkNameEl = document.getElementById('network-name');
        if (networkNameEl) {
            networkNameEl.textContent = this.mesh && this.mesh.currentNetwork || 'None';
        }
        
        // Update connection status with styling
        this.updateConnectionStatus(connectionStatus);
    }

    updateConnectionStatus(connectionStatus) {
        const statusElement = document.getElementById('status');
        const connectionDiv = document.getElementById('connection-status');
        
        if (!statusElement || !connectionDiv) return;
        
        // Remove all status classes
        connectionDiv.className = 'status-item';
        
        switch (connectionStatus.status) {
            case 'fully_connected':
                statusElement.textContent = `Connected (${connectionStatus.connections} P2P)`;
                connectionDiv.classList.add('connected-indicator');
                break;
            case 'p2p_only':
                statusElement.textContent = `P2P Only (${connectionStatus.connections})`;
                connectionDiv.classList.add('p2p-indicator');
                break;
            case 'tracker_only':
                statusElement.textContent = 'Tracker Only';
                connectionDiv.classList.add('connected-indicator');
                break;
            default:
                statusElement.textContent = 'Offline';
                connectionDiv.classList.add('offline-indicator');
        }
    }

    updateButtonStates(connectionStatus) {
        const hasValidators = this.blockchain.get_validator_count() > 0;
        const connected = this.mesh && (this.mesh.connected || this.mesh.dataChannels.size > 0);
        const hasNetwork = this.mesh && this.mesh.currentNetwork;
        
        // Connection button
        const connectBtn = document.getElementById('connect-btn');
        if (connectBtn) {
            connectBtn.textContent = this.mesh && this.mesh.connected ? 'Disconnect' : 'Connect';
        }
        
        // Network buttons
        const joinNetworkBtn = document.getElementById('join-network-btn');
        if (joinNetworkBtn) {
            joinNetworkBtn.disabled = !connected;
        }
        
        const discoverBtn = document.getElementById('discover-btn');
        if (discoverBtn) {
            discoverBtn.disabled = !hasNetwork;
        }
        
        const connectAllBtn = document.getElementById('connect-all-btn');
        if (connectAllBtn) {
            connectAllBtn.disabled = !hasNetwork || !this.mesh?.availablePeers.length;
        }
        
        // Messaging button
        const messageBtn = document.getElementById('message-btn');
        if (messageBtn) {
            messageBtn.disabled = !connected;
            
            // Add visual indication for offline capability
            if (connectionStatus.status === 'p2p_only') {
                messageBtn.classList.add('offline-enabled');
                messageBtn.title = 'Messaging works via P2P even when tracker is offline';
            } else {
                messageBtn.classList.remove('offline-enabled');
                messageBtn.title = '';
            }
        }
    }

    ensureValidators(connectionStatus) {
        const hasValidators = this.blockchain.get_validator_count() > 0;
        const connected = this.mesh && (this.mesh.connected || this.mesh.dataChannels.size > 0);
        
        if (connected && !hasValidators) {
            try {
                this.blockchain.add_validator(this.getUserId(), 1000);
                console.log('Auto-added validator for PoS consensus');
            } catch (e) {
                // Validator might already exist
            }
        }
    }

    updateBlockchain() {
        if (!this.blockchain) return;
        
        // Update blockchain display in all tabs
        const blockchainElements = [
            'blockchain',           // messaging tab
            'blockchain-trading',   // trading tab
            'blockchain-orderbook', // order book tab
            'blockchain-contracts', // smart contracts tab
            'blockchain-editor'     // contract editor tab
        ];
        
        blockchainElements.forEach(elementId => {
            const blockchainDiv = document.getElementById(elementId);
            if (blockchainDiv) {
                this.updateBlockchainElement(blockchainDiv, elementId);
            }
        });
    }

    updateBlockchainElement(blockchainDiv, elementId) {
        try {
            const chainLength = this.blockchain.get_chain_length();
            
            // Get recent blocks (last 5 blocks)
            let html = this.generateBlockchainHeader(chainLength);
            
            // Show recent blocks with single transactions
            for (let i = Math.max(1, chainLength - 4); i < chainLength; i++) {
                const blockJson = this.blockchain.get_latest_block_json();
                if (blockJson && blockJson !== '{}') {
                    const block = JSON.parse(blockJson);
                    if (block.height === i) {
                        html += this.generateBlockDisplay(block, elementId);
                    }
                }
            }
            
            blockchainDiv.innerHTML = html;
            
        } catch (error) {
            console.error('Error updating blockchain display:', error);
            blockchainDiv.innerHTML = '<p style="color: #dc3545;">Error loading blockchain data</p>';
        }
    }

    generateBlockchainHeader(chainLength) {
        const pendingCount = this.blockchain.get_pending_count();
        
        return `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <div>
                    <h4 style="margin: 0; color: #007bff;">Blockchain Status</h4>
                    <p style="margin: 5px 0; color: #666;">
                        One block per message/transaction
                    </p>
                </div>
                <div style="text-align: right;">
                    <div style="background: #e7f3ff; padding: 8px 12px; border-radius: 6px; margin-bottom: 5px;">
                        <strong>${chainLength - 1}</strong> Blocks
                    </div>
                    <div style="background: ${pendingCount > 0 ? '#fff3cd' : '#d4edda'}; padding: 8px 12px; border-radius: 6px;">
                        <strong>${pendingCount}</strong> Pending
                    </div>
                </div>
            </div>
        `;
    }

    generateBlockDisplay(block, elementId) {
        if (!block || !block.transactions || block.transactions.length === 0) {
            return '';
        }

        // Each block should have only one transaction
        const tx = block.transactions[0];
        let content = '';
        let typeClass = 'block-display';
        let typeLabel = 'Transaction';
        
        // Filter content based on tab context
        const isRelevantForTab = this.isTransactionRelevantForTab(tx, elementId);
        if (!isRelevantForTab) {
            return '';
        }
        
        if (tx.tx_type && tx.tx_type.Message) {
            content = tx.tx_type.Message.content;
            typeClass = 'block-display message-block';
            typeLabel = 'Message';
        } else if (tx.tx_type && tx.tx_type.Trading) {
            const trading = tx.tx_type.Trading;
            content = `${trading.quantity / 100} ${trading.asset} @ $${trading.price / 100}`;
            typeClass = 'block-display trading-block';
            typeLabel = 'Trade';
        } else if (tx.tx_type === 'Transfer') {
            content = `${tx.amount} units`;
            typeClass = 'block-display transfer-block';
            typeLabel = 'Transfer';
        }

        return `
            <div class="${typeClass}" style="margin: 15px 0; padding: 15px; border-radius: 8px; border-left: 4px solid #007bff;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                    <div>
                        <span style="background: #007bff; color: white; padding: 2px 8px; border-radius: 12px; font-size: 11px;">
                            ${typeLabel}
                        </span>
                        <span style="margin-left: 10px; font-weight: bold;">Block #${block.height}</span>
                    </div>
                    <div style="font-size: 12px; color: #666;">
                        ${new Date(block.timestamp * 1000).toLocaleTimeString()}
                    </div>
                </div>
                
                <div style="background: #f8f9fa; padding: 10px; border-radius: 6px; margin-bottom: 10px;">
                    <strong>"${content}"</strong>
                </div>
                
                <div style="font-size: 12px; color: #666; display: flex; justify-content: space-between;">
                    <span><strong>From:</strong> ${tx.from?.substring(0, 12) || 'unknown'}...</span>
                    <span><strong>Amount:</strong> ${tx.amount || 0}</span>
                    <span><strong>Validator:</strong> ${block.validator?.substring(0, 12) || 'genesis'}...</span>
                </div>
            </div>
        `;
    }

    isTransactionRelevantForTab(tx, elementId) {
        // Show all transactions in messaging tab
        if (elementId === 'blockchain') {
            return true;
        }
        
        // Show only trading transactions in trading and orderbook tabs
        if (elementId === 'blockchain-trading' || elementId === 'blockchain-orderbook') {
            return tx.tx_type && tx.tx_type.Trading;
        }
        
        // Show contract transactions in contracts tab
        if (elementId === 'blockchain-contracts') {
            return tx.tx_type && (tx.tx_type.ContractDeploy || tx.tx_type.ContractCall);
        }
        
        // Show contract-related transactions in editor tab
        if (elementId === 'blockchain-editor') {
            return tx.tx_type && (tx.tx_type.ContractDeploy || tx.tx_type.ContractCall);
        }
        
        return true;
    }

    sendMessage() {
        const messageText = document.getElementById('message-input')?.value.trim();
        if (!messageText) return;

        if (this.mesh && typeof this.mesh.sendMessage === 'function') {
            const success = this.mesh.sendMessage(messageText);
            if (success) {
                const messageInput = document.getElementById('message-input');
                if (messageInput) messageInput.value = '';
                
                this.updateUI();
            } else {
                this.showAlert('Failed to send message', 'error');
            }
        } else {
            // Fallback to direct blockchain add (auto-mines)
            try {
                this.blockchain.add_message(messageText, this.getUserId());
                console.log('Message added to blockchain:', messageText);
                
                const messageInput = document.getElementById('message-input');
                if (messageInput) messageInput.value = '';
                
                this.updateUI();
            } catch (error) {
                console.error('Error sending message:', error);
                this.showAlert('Failed to send message', 'error');
            }
        }
    }

    mineBlock() {
        // Mining is now automatic when transactions are added
        console.log('Mining is automatic - no manual mining needed');
        this.showAlert('Mining is automatic in this system', 'info');
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
        const selectedTab = event?.target;
        
        if (selectedPanel) selectedPanel.classList.add('active');
        if (selectedTab) selectedTab.classList.add('active');
        
        // Update blockchain display when switching tabs
		if (window.blockchainUI && window.blockchainUI.updateBlockchain) {
			window.blockchainUI.updateBlockchain();
		}
    }

    showAlert(message, type = 'info') {
        let alert = document.getElementById('ui-alert');
        if (!alert) {
            alert = document.createElement('div');
            alert.id = 'ui-alert';
            alert.style.cssText = `
                position: fixed;
                bottom: 20px;
                right: 20px;
                padding: 12px 20px;
                border-radius: 6px;
                z-index: 1000;
                font-size: 14px;
                max-width: 300px;
                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            `;
            document.body.appendChild(alert);
        }
        
        // Set style based on type
        switch (type) {
            case 'success':
                alert.style.background = '#d4edda';
                alert.style.color = '#155724';
                alert.style.border = '1px solid #c3e6cb';
                break;
            case 'error':
                alert.style.background = '#f8d7da';
                alert.style.color = '#721c24';
                alert.style.border = '1px solid #f5c6cb';
                break;
            case 'warning':
                alert.style.background = '#fff3cd';
                alert.style.color = '#856404';
                alert.style.border = '1px solid #ffeaa7';
                break;
            default:
                alert.style.background = '#d1ecf1';
                alert.style.color = '#0c5460';
                alert.style.border = '1px solid #bee5eb';
        }
        
        alert.textContent = message;
        alert.style.display = 'block';
        
        setTimeout(() => {
            if (alert.parentNode) {
                alert.style.display = 'none';
            }
        }, 3000);
    }

    startUpdateLoop() {
        // Update UI every 2 seconds
        this.updateInterval = setInterval(() => {
            this.updateUI();
        }, 2000);
    }

    stopUpdateLoop() {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
            this.updateInterval = null;
        }
    }

    getUserId() {
        if (!window.userId) {
            window.userId = 'user_' + Math.random().toString(36).substr(2, 9);
        }
        return window.userId;
    }

    destroy() {
        this.stopUpdateLoop();
        
        // Remove event listeners
        window.removeEventListener('wasmReady', this.updateUI);
        window.removeEventListener('meshReady', this.updateUI);
        window.removeEventListener('meshUpdate', this.updateUI);
        window.removeEventListener('tradingUpdate', this.updateUI);
    }
}

// Create global UI manager instance
const blockchainUI = new BlockchainUI();

// Export functions for global access
window.sendMessage = blockchainUI.sendMessage;
window.mineBlock = blockchainUI.mineBlock;
window.showTab = blockchainUI.showTab;
window.updateUI = blockchainUI.updateUI;
window.blockchainUI = blockchainUI;

export { blockchainUI, BlockchainUI };

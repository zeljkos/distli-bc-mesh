// Updated Blockchain Core - Compatible with Leadership Manager
// File: public/js/blockchain-core.js

class SmartBlockchain {
    constructor() {
        this.chain = [this.createGenesis()];
        this.pending = [];
        this.contract_vm = new ContractVM();
        this.processed_transaction_ids = new Set();
        this.sent_blockchain_updates = new Set();
        
        // Leadership integration
        this.isMiningLeader = false; // Controlled by leadership manager
        this.useRobustLeadership = false; // Flag for leadership manager integration
        
        // Transaction processing
        this.blockProcessingLock = false;
        this.transactionProcessingQueue = [];
        
        this.initializeTradingContract();
        
        // Only start legacy leadership if robust leadership is not being used
        setTimeout(() => {
            if (!this.useRobustLeadership) {
                this.startLegacyLeadershipElection();
            }
        }, 1000);
    }
    
    createGenesis() {
        return {
            id: 0,
            hash: "000genesis",
            prev_hash: "0",
            timestamp: Date.now(),
            data: "Genesis Block",
            nonce: 0,
            transactions: []
        };
    }

    // Legacy leadership election (only runs if robust leadership is not active)
    startLegacyLeadershipElection() {
        if (this.useRobustLeadership) {
            console.log('🔇 Skipping legacy leadership - robust system active');
            return;
        }
        
        // Wait for network connectivity
        if (!window.mesh || !window.mesh.connected || !window.mesh.currentNetwork) {
            setTimeout(() => this.startLegacyLeadershipElection(), 1000);
            return;
        }

        console.log('🗳️ Starting legacy leader election for mining coordination');
        this.checkLegacyLeadershipStatus();
        
        // Periodic leader election check every 10 seconds
        setInterval(() => {
            if (!this.useRobustLeadership) {
                this.checkLegacyLeadershipStatus();
            }
        }, 10000);
    }

    checkLegacyLeadershipStatus() {
        if (this.useRobustLeadership) return;
        
        if (!window.mesh || !window.mesh.currentNetwork) return;

        const networkId = window.mesh.currentNetwork;
        const myPeerId = getUserId();
        
        // Get connected peers
        const connectedPeers = window.mesh.dataChannels ? Array.from(window.mesh.dataChannels.keys()) : [];
        const allPeers = [myPeerId, ...connectedPeers].sort();
        
        // Simplified leadership logic for legacy mode
        const wasLeader = this.isMiningLeader;
        const shouldBeLeader = (allPeers.length === 1) || (allPeers[0] === myPeerId);
        
        if (shouldBeLeader && !wasLeader) {
            this.isMiningLeader = true;
            console.log(`👑 Legacy leader elected for network ${networkId}`);
        } else if (!shouldBeLeader && wasLeader && allPeers.length > 2) {
            this.isMiningLeader = false;
            console.log(`👥 Stepped down from legacy leadership for network ${networkId}`);
        }
        
        if (allPeers.length <= 2 && !this.isMiningLeader) {
            console.log('🔧 Only 2 peers - becoming legacy leader');
            this.isMiningLeader = true;
        }
        
        console.log(`👥 Legacy Network ${networkId}: ${allPeers.length} peers, leader: ${this.isMiningLeader ? 'ME' : 'OTHER'}`);
    }

    // Enhanced transaction existence checking
    transactionExists(transactionId) {
        if (this.processed_transaction_ids.has(transactionId)) {
            return true;
        }

        const inPending = this.pending.some(tx => tx.id === transactionId);
        if (inPending) {
            this.processed_transaction_ids.add(transactionId);
            return true;
        }

        for (const block of this.chain) {
            if (block.transactions && block.transactions.some(tx => tx.id === transactionId)) {
                this.processed_transaction_ids.add(transactionId);
                return true;
            }
        }

        return false;
    }
    
    initializeTradingContract() {
        this.contract_vm = new ContractVM();
        console.log('Multi-language contract system initialized');
    }
    
    // Enhanced message handling with leadership coordination
    addMessage(data, sender = 'anonymous', skipMiningPrompt = false) {
        const tx = {
            type: 'message',
            id: 'msg_' + Date.now() + Math.random().toString(36).substr(2, 9),
            data: data,
            timestamp: Date.now(),
            sender: sender
        };

        if (this.transactionExists(tx.id)) {
            console.log(`Transaction ${tx.id} already exists, skipping`);
            return tx;
        }

        this.pending.push(tx);
        this.processed_transaction_ids.add(tx.id);
        this.saveToStorage();

        console.log(`📝 Added message transaction: ${data} (pending: ${this.pending.length}, leader: ${this.isMiningLeader})`);

        // Mining prompt logic
        if (!skipMiningPrompt && this.pending.length === 1) {
            if (this.isMiningLeader) {
                // Auto-mine for leaders, prompt for legacy mode
                if (this.useRobustLeadership) {
                    console.log(`💡 Transaction added - robust leader will auto-mine`);
                } else {
                    setTimeout(() => {
                        if (this.pending.length > 0 && this.isMiningLeader) {
                            if (confirm('Mine block to include your transaction?')) {
                                this.mineBlock();
                            }
                        }
                    }, 500);
                }
            } else {
                console.log(`💡 Transaction added - waiting for leader to mine`);
                log(`Transaction queued - mining leader will process it`);
            }
        }
        
        return tx;
    }

    // Enhanced contract call handling
    call_contract(call, sender, skipMiningPrompt = false) {
        console.log(`🔧 call_contract: ${call.function} by ${sender}, leader: ${this.isMiningLeader}`);
        
        const timestamp = Date.now();
        const random = Math.random().toString(36).substr(2, 5);
        const txId = `call_${call.function}_${timestamp}_${random}`;
        
        // Simplified duplicate checking
        const recentTxs = this.pending.slice(-10);
        const isDuplicate = recentTxs.some(tx => 
            tx.type === 'contract_call' && 
            tx.call.function === call.function &&
            tx.call.contract_id === call.contract_id &&
            JSON.stringify(tx.call.params) === JSON.stringify(call.params) &&
            tx.sender === sender &&
            (timestamp - tx.timestamp) < 5000
        );
        
        if (isDuplicate) {
            console.log(`Recent duplicate contract call detected, skipping`);
            return null;
        }

        call.caller = sender;
        const result = this.contract_vm.call_contract(call);
        
        const transaction = {
            type: 'contract_call',
            id: txId,
            call: call,
            result: result,
            timestamp: timestamp,
            sender: sender
        };
                
        this.pending.push(transaction);
        this.processed_transaction_ids.add(txId);
        this.saveToStorage();
        
        console.log(`📝 Contract call added: ${call.function} - Result: ${result.success ? 'SUCCESS' : 'FAILED'}`);
                
        // Enhanced auto-mining for leaders
        if (!skipMiningPrompt && this.isMiningLeader) {
            console.log(`⛏️ Auto-mining as leader...`);
            setTimeout(() => {
                if (this.pending.length > 0 && this.isMiningLeader) {
                    const block = this.mineBlock();
                    if (block) {
                        console.log(`✅ Auto-mined block #${block.id}`);
                        this.broadcastNewBlock(block);
                        if (typeof updateUI === 'function') updateUI();
                        if (typeof updateOrderBookFromContract === 'function') updateOrderBookFromContract();
                    }
                }
            }, 200);
        }
        
        return transaction;
    }

    // Transaction handling from peers with better deduplication
    addTransactionFromPeer(transaction) {
        const existsInPending = this.pending.some(tx => tx.id === transaction.id);
        const existsInChain = this.chain.some(block => 
            block.transactions && block.transactions.some(tx => tx.id === transaction.id)
        );
        
        if (existsInPending || existsInChain) {
            console.log(`Received duplicate transaction ${transaction.id}, ignoring`);
            return false;
        }

        console.log(`📨 Received transaction from peer: ${transaction.type} (${transaction.id})`);

        if (transaction.type === 'message') {
            this.pending.push(transaction);
            this.processed_transaction_ids.add(transaction.id);
        } else if (transaction.type === 'contract_call') {
            if (transaction.result && transaction.result.success) {
                this.contract_vm.apply_state_changes(
                    transaction.call.contract_id,
                    transaction.result.state_changes
                );
            }
            this.pending.push(transaction);
            this.processed_transaction_ids.add(transaction.id);
        } else if (transaction.type === 'cross_network_trade') {
            this.pending.push(transaction);
            this.processed_transaction_ids.add(transaction.id);
        }

        this.saveToStorage();
        
        // Auto-mine if we're leader with pending transactions
        if (this.isMiningLeader && this.pending.length > 0) {
            console.log(`⛏️ Auto-mining peer transaction as leader`);
            setTimeout(() => {
                if (this.pending.length > 0 && this.isMiningLeader) {
                    this.mineBlock();
                }
            }, 1000);
        }
        
        return true;
    }

    // Emergency mining for deadlock resolution
    forceMineAnyPending() {
        console.log('🚨 EMERGENCY MINING - Mining any pending transactions');

        if (this.pending.length === 0) {
            console.log('No pending transactions to emergency mine');
            return null;
        }

        // Temporarily become leader
        const wasLeader = this.isMiningLeader;
        this.isMiningLeader = true;

        try {
            const block = this.mineBlock();
            if (block) {
                console.log(`✅ Emergency mined block #${block.id} with ${block.transactions.length} transactions`);

                if (typeof updateUI === 'function') updateUI();
                if (typeof updateOrderBookFromContract === 'function') updateOrderBookFromContract();

                return block;
            }
        } finally {
            // Restore original leader status
            this.isMiningLeader = wasLeader;
        }

        return null;
    }

    // Cross-network trade handling
    addCrossNetworkTrade(message, isInitiator = false) {
        console.log(`🔍 addCrossNetworkTrade called - isInitiator: ${isInitiator}`);
        
        if (!isInitiator) {
            console.log(`📝 Cross-network trade notification received - updating local state only`);
            
            const tradeId = message.trade_id || `cross_${message.buyer_network}_${message.seller_network}_${Date.now()}`;
            this.processed_transaction_ids.add(tradeId);
            this.saveToStorage();
            
            return null;
        }
        
        console.log(`⚠️ addCrossNetworkTrade called as initiator`);
        
        const tradeId = message.trade_id || `cross_${message.buyer_network}_${message.seller_network}_${Date.now()}`;

        if (this.transactionExists(tradeId)) {
            console.log(`Cross-network trade ${tradeId} already processed, skipping`);
            return null;
        }

        const crossNetworkTransaction = {
            type: 'cross_network_trade',
            id: tradeId,
            data: `Cross-network trade: ${message.quantity} ${message.asset} @ ${message.price}`,
            buyer_network: message.buyer_network,
            seller_network: message.seller_network,
            asset: message.asset,
            quantity: message.quantity,
            price: message.price,
            timestamp: message.timestamp * 1000,
            sender: 'cross_network_matcher'
        };

        this.pending.push(crossNetworkTransaction);
        this.processed_transaction_ids.add(tradeId);
        this.saveToStorage();

        return crossNetworkTransaction;
    }
    
    get_order_book(asset = null) {
        try {
            return this.contract_vm.get_order_book(asset);
        } catch (error) {
            console.log('Error getting order book: ' + error.message);
            return { bids: [], asks: [] };
        }
    }
    
    get_recent_trades(asset = null, limit = 10) {
        try {
            return this.contract_vm.get_recent_trades(asset, limit);
        } catch (error) {
            console.log('Error getting recent trades: ' + error.message);
            return { trades: [] };
        }
    }
    
    // Enhanced mining with proper leadership checks
    mineBlock() {
        if (this.pending.length === 0) {
            console.log('📭 No pending transactions to mine');
            return null;
        }

        // Check leadership status
        if (!this.isMiningLeader) {
            console.log('❌ Mining rejected - not authorized as leader');
            return null;
        }

        console.log(`⛏️ Mining block with ${this.pending.length} transactions`);
        
        try {
            const lastBlock = this.chain[this.chain.length - 1];
            const transactions = [...this.pending];
            
            const data = transactions.map(tx => {
                if (tx.type === 'message') {
                    return `${tx.sender}: ${tx.data}`;
                } else if (tx.type === 'contract_call') {
                    const asset = tx.call.params.asset || 'unknown';
                    const qty = tx.call.params.quantity || '';
                    const price = tx.call.params.price || '';
                    return `${tx.sender}: ${tx.call.function}(${asset} ${qty}@${price})`;
                }
                return `${tx.sender}: ${tx.type}`;
            }).join(", ");
            
            const block = {
                id: lastBlock.id + 1,
                hash: "000" + Date.now().toString(),
                prev_hash: lastBlock.hash,
                timestamp: Date.now(),
                data: data,
                nonce: Math.floor(Math.random() * 1000000),
                transactions: transactions,
                miner: getUserId()
            };
            
            this.chain.push(block);
            this.pending = []; // Clear all pending
            this.saveToStorage();
            
            console.log(`✅ Block #${block.id} mined successfully by ${this.useRobustLeadership ? 'verified' : 'legacy'} leader`);
            
            this.sendBlockchainUpdate();
            this.broadcastNewBlock(block);
            
            return block;
            
        } catch (error) {
            console.error('❌ Mining error:', error);
            return null;
        }
    }

    // Broadcast new block to peers
    broadcastNewBlock(block) {
        if (window.mesh && window.mesh.connected) {
            const blockMessage = { 
                type: 'block', 
                block: block,
                miner: getUserId(),
                timestamp: Date.now()
            };
            
            try {
                window.mesh.broadcast(blockMessage);
                console.log(`📡 Broadcasted block #${block.id} to network`);
            } catch (error) {
                console.error('Failed to broadcast block:', error);
            }
        }
    }
    
    sendBlockchainUpdate() {
        if (!window.mesh || !window.mesh.connected || !window.mesh.currentNetwork) {
            console.log('Cannot send blockchain update - not connected to network');
            return;
        }
        
        const recentBlocks = this.chain.slice(1).slice(-5).map(block => ({
            block_id: block.id,
            block_hash: block.hash,
            transactions: block.transactions.map(tx => {
                if (tx.type === 'message') {
                    return tx.data;
                } else if (tx.type === 'contract_call') {
                    return `${tx.call.function}: ${JSON.stringify(tx.call.params)}`;
                }
                return tx.type;
            }),
            timestamp: Math.floor(block.timestamp / 1000),
            previous_hash: block.prev_hash,
            miner: block.miner || getUserId()
        }));
        
        if (recentBlocks.length === 0) {
            console.log('No blocks to send to enterprise BC');
            return;
        }
        
        const updateId = `${window.mesh.currentNetwork}_${getUserId()}_${recentBlocks[recentBlocks.length - 1].block_id}`;
        
        if (this.sent_blockchain_updates.has(updateId)) {
            console.log(`⚠️ Blockchain update ${updateId} already sent, skipping duplicate`);
            return;
        }
        
        const message = {
            type: 'blockchain_update',
            network_id: window.mesh.currentNetwork,
            peer_id: getUserId(),
            new_blocks: recentBlocks,
            timestamp: Math.floor(Date.now() / 1000),
            is_leader: this.isMiningLeader
        };
        
        window.mesh.send(message);
        this.sent_blockchain_updates.add(updateId);
        
        // Clean up old update IDs
        if (this.sent_blockchain_updates.size > 50) {
            const oldestIds = Array.from(this.sent_blockchain_updates).slice(0, 10);
            oldestIds.forEach(id => this.sent_blockchain_updates.delete(id));
        }
        
        console.log(`📤 Sent blockchain update: ${recentBlocks.length} blocks to tracker (ID: ${updateId})`);
    }
    
    // Enhanced block addition with better validation
    addBlock(block) {
        const existingBlock = this.chain.find(b => b.id === block.id);
        if (existingBlock) {
            if (existingBlock.hash === block.hash) {
                console.log(`Block #${block.id} already exists with same hash, skipping`);
                return false;
            } else {
                console.log(`⚠️ Block #${block.id} conflict - different hash!`);
                return false;
            }
        }

        const lastBlock = this.chain[this.chain.length - 1];
        
        if (block.id === lastBlock.id + 1 && block.prev_hash === lastBlock.hash) {
            for (const tx of block.transactions) {
                if (tx.type === 'contract_call' && tx.result) {
                    this.contract_vm.apply_state_changes(tx.call.contract_id, tx.result.state_changes);
                }
                this.processed_transaction_ids.add(tx.id);
            }
            
            this.chain.push(block);
            
            const blockTxIds = new Set(block.transactions.map(tx => tx.id));
            this.pending = this.pending.filter(pendingTx => !blockTxIds.has(pendingTx.id));
            
            this.saveToStorage();
            
            console.log(`✅ Added block #${block.id} from network (miner: ${block.miner || 'unknown'})`);
            return true;
        } else {
            console.log(`❌ Invalid block #${block.id} - chain validation failed`);
            return false;
        }
    }
    
    saveToStorage() {
        try {
            const data = {
                chain: this.chain,
                pending: this.pending,
                contract_state: this.contract_vm.get_all_state(),
                processed_transaction_ids: Array.from(this.processed_transaction_ids),
                sent_blockchain_updates: Array.from(this.sent_blockchain_updates),
                is_mining_leader: this.isMiningLeader,
                use_robust_leadership: this.useRobustLeadership,
                lastSaved: Date.now()
            };
            const networkId = (window.mesh && window.mesh.currentNetwork) || 'default';
            localStorage.setItem(`blockchain_${networkId}`, JSON.stringify(data));
        } catch (error) {
            console.log(`Failed to save blockchain: ${error.message}`);
        }
    }
    
    loadFromStorage() {
        try {
            const networkId = (window.mesh && window.mesh.currentNetwork) || 'default';
            const stored = localStorage.getItem(`blockchain_${networkId}`);
            if (stored) {
                const data = JSON.parse(stored);
                if (data && data.chain && data.chain.length > 0) {
                    this.chain = data.chain;
                    this.pending = data.pending || [];
                    
                    if (data.contract_state) {
                        this.contract_vm.restore_state(data.contract_state);
                    } else {
                        this.initializeTradingContract();
                    }
                    
                    if (data.processed_transaction_ids) {
                        this.processed_transaction_ids = new Set(data.processed_transaction_ids);
                    }
                    
                    if (data.sent_blockchain_updates) {
                        this.sent_blockchain_updates = new Set(data.sent_blockchain_updates);
                    }
                    
                    // Restore leadership flags
                    this.useRobustLeadership = data.use_robust_leadership || false;
                    
                    console.log(`📚 Loaded blockchain for ${networkId}: ${this.chain.length} blocks, ${this.pending.length} pending, robust leadership: ${this.useRobustLeadership}`);
                    return true;
                }
            }
            this.initializeTradingContract();
            return false;
        } catch (error) {
            console.log(`Failed to load blockchain: ${error.message}`);
            this.initializeTradingContract();
            return false;
        }
    }

    // Enable robust leadership integration
    enableRobustLeadership() {
        console.log('🔧 Enabling robust leadership integration');
        this.useRobustLeadership = true;
        this.isMiningLeader = false; // Will be controlled by leadership manager
    }

    // Cleanup method for proper resource management
    destroy() {
        this.processed_transaction_ids.clear();
        this.sent_blockchain_updates.clear();
        this.useRobustLeadership = false;
        this.isMiningLeader = false;
    }
}

// Utility functions
function getUserId() {
    if (!window.userId) {
        window.userId = 'user_' + Math.random().toString(36).substr(2, 9);
    }
    return window.userId;
}

function log(message) {
    const logDiv = document.getElementById('log');
    if (logDiv) {
        const time = new Date().toLocaleTimeString();
        logDiv.innerHTML += time + ': ' + message + '\n';
        logDiv.scrollTop = logDiv.scrollHeight;
    }
    console.log(message);
}

// Enhanced debug functions
function debugOrderBook() {
    if (!window.blockchain) {
        console.log('❌ No blockchain instance found');
        return;
    }
    
    console.log('🔍 DEBUG: Current Order Book State');
    console.log('=====================================');
    console.log(`👥 Network: ${window.mesh ? window.mesh.currentNetwork : 'Unknown'}`);
    console.log(`👑 Mining Leader: ${window.blockchain.isMiningLeader ? 'YES' : 'NO'}`);
    console.log(`🏛️ Robust Leadership: ${window.blockchain.useRobustLeadership ? 'YES' : 'NO'}`);
    console.log(`📊 Connected Peers: ${window.mesh ? window.mesh.dataChannels.size : 0}`);
    
    const orderBook = window.blockchain.get_order_book();
    console.log('📖 Order Book:', orderBook);
    
    const tradingContract = window.blockchain.contract_vm.contracts.get("trading_contract");
    if (tradingContract) {
        console.log('📋 Trading Contract State:', tradingContract.state);
    }
    
    console.log('⏳ Pending Transactions:', window.blockchain.pending);
    console.log('🔗 Pending Count:', window.blockchain.pending.length);
    
    window.blockchain.pending.forEach((tx, i) => {
        console.log(`  📄 Pending ${i+1}:`, {
            type: tx.type,
            id: tx.id,
            sender: tx.sender,
            timestamp: new Date(tx.timestamp).toLocaleTimeString(),
            details: tx.call ? `${tx.call.function}(${JSON.stringify(tx.call.params)})` : tx.data
        });
    });
    
    console.log('🔒 Processed Transaction IDs:', Array.from(window.blockchain.processed_transaction_ids).slice(-10));
    console.log('=====================================');
}

function clearPendingTransactions() {
    if (!window.blockchain) {
        console.log('❌ No blockchain instance found');
        return;
    }
    
    const count = window.blockchain.pending.length;
    console.log(`🧹 About to clear ${count} pending transactions`);
    
    window.blockchain.pending.forEach((tx, i) => {
        console.log(`  🗑️ Clearing ${i+1}:`, {
            type: tx.type,
            details: tx.call ? `${tx.call.function}(${JSON.stringify(tx.call.params)})` : tx.data
        });
    });
    
    window.blockchain.pending = [];
    window.blockchain.saveToStorage();
    
    console.log(`✅ Cleared ${count} pending transactions`);
    
    if (typeof updateUI === 'function') {
        updateUI();
    }
}

function clearPendingByAsset(asset, orderType) {
    if (!window.blockchain) {
        console.log('❌ No blockchain instance found');
        return;
    }
    
    const originalCount = window.blockchain.pending.length;
    
    window.blockchain.pending = window.blockchain.pending.filter(tx => {
        if (tx.type !== 'contract_call') return true;
        
        const call = tx.call;
        if (!call || call.contract_id !== 'trading_contract') return true;
        
        const isMatch = call.function === orderType && call.params.asset === asset;
        if (isMatch) {
            console.log(`🗑️ Removing: ${call.function}(${JSON.stringify(call.params)})`);
        }
        return !isMatch;
    });
    
    const removedCount = originalCount - window.blockchain.pending.length;
    console.log(`✅ Removed ${removedCount} pending ${orderType} transactions for ${asset}`);
    
    window.blockchain.saveToStorage();
    if (typeof updateUI === 'function') {
        updateUI();
    }
}

// Network monitoring functions
function monitorNetworkHealth() {
    if (!window.blockchain || !window.mesh) return;
    
    const healthInfo = {
        network: window.mesh.currentNetwork,
        connected: window.mesh.connected,
        peers: window.mesh.dataChannels ? window.mesh.dataChannels.size : 0,
        is_leader: window.blockchain.isMiningLeader,
        robust_leadership: window.blockchain.useRobustLeadership,
        pending_transactions: window.blockchain.pending.length,
        blockchain_height: window.blockchain.chain.length,
        processed_messages: window.mesh ? window.mesh.processedMessages?.size || 0 : 0
    };
    
    console.log('🏥 Network Health Check:', healthInfo);
    
    // Alerts for potential issues
    if (window.blockchain.pending.length > 10) {
        console.warn(`⚠️ High pending transactions (${window.blockchain.pending.length}) - possible coordination issue`);
    }
    
    if (window.mesh.dataChannels && window.mesh.dataChannels.size === 0 && window.mesh.currentNetwork) {
        console.warn('⚠️ No connected peers - check network connectivity');
    }
    
    return healthInfo;
}

// Make debug functions globally available
window.debugOrderBook = debugOrderBook;
window.clearPendingTransactions = clearPendingTransactions;
window.clearPendingByAsset = clearPendingByAsset;
window.monitorNetworkHealth = monitorNetworkHealth;

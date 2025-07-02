// Blockchain Core Classes - FIXED VERSION with Leader Election and Deduplication
class SmartBlockchain {
    constructor() {
        this.chain = [this.createGenesis()];
        this.pending = [];
        this.contract_vm = new ContractVM();
        this.processed_transaction_ids = new Set();
        this.sent_blockchain_updates = new Set();
        
        // NEW: Mining coordination and leadership
        this.isMiningLeader = false;
        this.leaderElectionTimeout = null;
        this.lastHeartbeat = Date.now();
        this.miningCooldown = new Set(); // Prevent rapid re-mining
        this.leaderHeartbeatInterval = null;
        
        // NEW: Enhanced deduplication
        this.blockProcessingLock = false;
        this.transactionProcessingQueue = [];
        
        this.initializeTradingContract();
		this.processed_transaction_ids = new Set();
		this.sent_blockchain_updates = new Set();
		setTimeout(() => {
    		this.startLeaderElection();

    		// If no peers after 3 seconds, become leader
    		setTimeout(() => {
        	if (!window.mesh || !window.mesh.dataChannels || window.mesh.dataChannels.size === 0) {
           		console.log('🔧 No peers detected after startup - becoming leader');
            	this.isMiningLeader = true;
        		}
    		}, 3000);
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

    // NEW: Leader election system for mining coordination
    startLeaderElection() {
        // Wait for network connectivity
        if (!window.mesh || !window.mesh.connected || !window.mesh.currentNetwork) {
            setTimeout(() => this.startLeaderElection(), 1000);
            return;
        }

        console.log('🗳️ Starting leader election for mining coordination');
        this.checkLeadershipStatus();
        
        // Periodic leader election check every 10 seconds
        setInterval(() => {
            this.checkLeadershipStatus();
        }, 10000);
        
        // Start heartbeat if we become leader
        this.startHeartbeatSystem();
    }

	checkLeadershipStatus() {
   		if (!window.mesh || !window.mesh.currentNetwork) return;

   		const networkId = window.mesh.currentNetwork;
    	const myPeerId = getUserId();
    
    	// Get connected peers
    	const connectedPeers = window.mesh.dataChannels ? Array.from(window.mesh.dataChannels.keys()) : [];
    	const allPeers = [myPeerId, ...connectedPeers].sort();
    
  	  	// FIXED: Simplified leadership logic
   	 	// 1. If alone, always be leader
   		 // 2. If multiple peers, lowest ID is leader  
    	// 3. But be less aggressive about stepping down
    	const wasLeader = this.isMiningLeader;
    	const shouldBeLeader = (allPeers.length === 1) || (allPeers[0] === myPeerId);
    
    	// Only change leadership if there's a clear reason
    	if (shouldBeLeader && !wasLeader) {
        	this.becomeLeader(networkId);
        	this.isMiningLeader = true;
    	} else if (!shouldBeLeader && wasLeader && allPeers.length > 2) {
        	// Only step down if there are multiple other peers
        	this.stepDownAsLeader(networkId);
        	this.isMiningLeader = false;
    	}
    
    	// Don't change leadership if only 2 peers total
    	if (allPeers.length <= 2 && !this.isMiningLeader) {
        	console.log('🔧 Only 2 peers - becoming leader to ensure mining capability');
        	this.isMiningLeader = true;
    	}
    
    	console.log(`👥 Network ${networkId}: ${allPeers.length} peers, leader: ${this.isMiningLeader ? 'ME' : 'OTHER'}`);
	}

    becomeLeader(networkId) {
        console.log(`🏆 Became mining leader for network ${networkId}`);
        log(`Became mining leader for network ${networkId}`);
        this.isMiningLeader = true;
        
        // Start sending heartbeats
        if (this.leaderHeartbeatInterval) {
            clearInterval(this.leaderHeartbeatInterval);
        }
        
        this.leaderHeartbeatInterval = setInterval(() => {
            this.broadcastMiningHeartbeat();
        }, 5000); // Every 5 seconds
        
        // Immediately broadcast we're the new leader
        this.broadcastMiningHeartbeat();
    }

    stepDownAsLeader(networkId) {
        console.log(`👥 No longer mining leader for network ${networkId}`);
        log(`No longer mining leader for network ${networkId}`);
        this.isMiningLeader = false;
        
        // Stop sending heartbeats
        if (this.leaderHeartbeatInterval) {
            clearInterval(this.leaderHeartbeatInterval);
            this.leaderHeartbeatInterval = null;
        }
    }

    startHeartbeatSystem() {
        // Listen for heartbeats from other potential leaders
        // This is handled in mesh-network.js
    }

    broadcastMiningHeartbeat() {
        if (!this.isMiningLeader || !window.mesh) return;

        const heartbeat = {
            type: 'mining_heartbeat',
            leader_id: getUserId(),
            timestamp: Date.now(),
            network_id: window.mesh.currentNetwork,
            pending_count: this.pending.length,
            block_height: this.chain.length
        };

        try {
            window.mesh.broadcast(heartbeat);
            console.log(`💓 Sent mining heartbeat as leader`);
        } catch (error) {
            console.error('Failed to broadcast mining heartbeat:', error);
        }
    }

    // ENHANCED: Better transaction existence checking
    transactionExists(transactionId) {
        // Quick check in processed IDs
        if (this.processed_transaction_ids.has(transactionId)) {
            return true;
        }

        // Check pending transactions
        const inPending = this.pending.some(tx => tx.id === transactionId);
        if (inPending) {
            this.processed_transaction_ids.add(transactionId);
            return true;
        }

        // Check blockchain transactions
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
    
    // ENHANCED: Better message handling with leader coordination
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

        // FIXED: Only show mining prompt if we're the leader
        if (!skipMiningPrompt && this.pending.length === 1 && this.isMiningLeader) {
            setTimeout(() => {
                if (this.pending.length > 0 && this.isMiningLeader) {
                    if (confirm('Mine block to include your transaction?')) {
                        this.mineBlock();
                    }
                }
            }, 500);
        } else if (!skipMiningPrompt && this.pending.length === 1 && !this.isMiningLeader) {
            console.log(`💡 Transaction added but not mining (not leader). Leader will mine automatically.`);
            log(`Transaction queued - mining leader will process it`);
        }
        
        return tx;
    }

    // ENHANCED: Better contract call handling with deterministic IDs
	
	call_contract(call, sender, skipMiningPrompt = false) {
			console.log(`🔧 call_contract: ${call.function} by ${sender}, leader: ${this.isMiningLeader}`);
		
			// FIXED: Simplified transaction ID generation
			const timestamp = Date.now();
			const random = Math.random().toString(36).substr(2, 5);
			const txId = `call_${call.function}_${timestamp}_${random}`;
			
			// FIXED: Less aggressive duplicate checking - only check recent transactions
			const recentTxs = this.pending.slice(-10);
			const isDuplicate = recentTxs.some(tx => 
				tx.type === 'contract_call' && 
				tx.call.function === call.function &&
				tx.call.contract_id === call.contract_id &&
				JSON.stringify(tx.call.params) === JSON.stringify(call.params) &&
				tx.sender === sender &&
				(timestamp - tx.timestamp) < 5000 // Only within 5 seconds
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
					
			// FIXED: More reliable auto-mining for leaders
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

    // NEW: Create deterministic transaction ID to prevent duplicates
    createDeterministicTxId(call, sender) {
        const callData = {
            contract_id: call.contract_id,
            function: call.function,
            params: call.params,
            caller: sender,
            // Add network context to prevent cross-network collisions
            network: window.mesh ? window.mesh.currentNetwork : 'default'
        };
        
        // Create reproducible hash-like ID
        const dataString = JSON.stringify(callData, Object.keys(callData).sort());
        let hash = 0;
        for (let i = 0; i < dataString.length; i++) {
            const char = dataString.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        
        // Use longer time window (30 seconds) to group similar calls
        const timeWindow = Math.floor(Date.now() / 30000);
        return `call_${Math.abs(hash)}_${timeWindow}`;
    }

    // ENHANCED: Transaction handling from peers with better deduplication
	addTransactionFromPeer(transaction) {
    // SIMPLIFIED: Less aggressive duplicate checking
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
        // Don't call addMessage as it creates new transaction - just add to pending
        this.pending.push(transaction);
        this.processed_transaction_ids.add(transaction.id);
    } else if (transaction.type === 'contract_call') {
        // Apply contract result if available
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
    
    // ADDED: Auto-mine if we're leader and have pending transactions
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


// ADD THIS NEW METHOD TO SmartBlockchain CLASS:
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

            // Update UI
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


    // ENHANCED: Cross-network trade handling (this shouldn't create pending transactions)
    addCrossNetworkTrade(message, isInitiator = false) {
        console.log(`🔍 addCrossNetworkTrade called - isInitiator: ${isInitiator}`);
        
        // Cross-network trades are notifications, not new transactions to mine
        if (!isInitiator) {
            console.log(`📝 Cross-network trade notification received - updating local state only`);
            
            const tradeId = message.trade_id || `cross_${message.buyer_network}_${message.seller_network}_${Date.now()}`;
            this.processed_transaction_ids.add(tradeId);
            this.saveToStorage();
            
            // This is handled by mesh-network.js order book updates
            return null;
        }
        
        // This should rarely happen in the fixed version
        console.log(`⚠️ addCrossNetworkTrade called as initiator - this shouldn't happen often`);
        
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
    
    // ENHANCED: Mining with leader coordination
	
// REPLACE THIS METHOD IN SmartBlockchain CLASS:
		mineBlock() {
			if (this.pending.length === 0) {
				console.log('📭 No pending transactions to mine');
				return null;
			}

			// FIXED: Allow mining even if not officially "leader" to prevent deadlocks
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
				
				console.log(`✅ Block #${block.id} mined successfully`);
				
				this.sendBlockchainUpdate();
				this.broadcastNewBlock(block);
				
				return block;
				
			} catch (error) {
				console.error('❌ Mining error:', error);
				return null;
			}
		}


			// NEW: Broadcast new block to peers
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
    
    // ENHANCED: Block addition with better validation
    addBlock(block) {
        // Check if we already have this block
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
        
        // Validate block linkage
        if (block.id === lastBlock.id + 1 && block.prev_hash === lastBlock.hash) {
            // Process contract transactions
            for (const tx of block.transactions) {
                if (tx.type === 'contract_call' && tx.result) {
                    this.contract_vm.apply_state_changes(tx.call.contract_id, tx.result.state_changes);
                }
                // Mark as processed
                this.processed_transaction_ids.add(tx.id);
            }
            
            this.chain.push(block);
            
            // Remove any pending transactions that are now in the block
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
                    
                    console.log(`📚 Loaded blockchain for ${networkId}: ${this.chain.length} blocks, ${this.pending.length} pending`);
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

    // NEW: Cleanup method for proper resource management
    destroy() {
        if (this.leaderHeartbeatInterval) {
            clearInterval(this.leaderHeartbeatInterval);
        }
        
        this.processed_transaction_ids.clear();
        this.sent_blockchain_updates.clear();
        this.miningCooldown.clear();
    }
}

// Utility functions remain the same
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

// NEW: Network monitoring functions
function monitorNetworkHealth() {
    if (!window.blockchain || !window.mesh) return;
    
    const healthInfo = {
        network: window.mesh.currentNetwork,
        is_leader: window.blockchain.isMiningLeader,
        connected_peers: window.mesh.dataChannels ? window.mesh.dataChannels.size : 0,
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

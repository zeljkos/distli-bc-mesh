// app.js - Updated with trade execution handling
import init, { Blockchain, OrderBook } from '../pkg/distli_mesh_bc.js';

class DistliApp {
    constructor() {
        this.blockchain = null;
        this.orderBook = null;
        this.ws = null;
        this.connected = false;
        this.currentNetwork = null;
        this.dataChannels = new Map();
        this.peers = new Map();
        this.availablePeers = [];
        this.userId = 'user_' + Math.random().toString(36).substr(2, 9);
        this.recentBlocks = [];
		this.remoteOrders = { bids: [], asks: [] };
        this.processedTrades = new Set(); // Track processed trade executions to prevent duplicates
        this.rtcConfig = {
            iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
        };
        
        // GSM Roaming wallet system
        this.wallets = new Map();
        this.initializeWallets();
    }

    async init() {
        try {
            await init();
            this.blockchain = new Blockchain();
            this.orderBook = new OrderBook();
            this.blockchain.add_validator(this.userId, 1000);
            
            this.setupEventListeners();
            this.setupDefaultServer();
            this.updateUI();
            
            setInterval(() => {
                this.updateUI();
                this.updateActiveSessionsDisplay(); // Update GSM roaming sessions display
                this.updateWalletDisplay(); // Update wallet balances
            }, 2000);
            
            console.log('App initialized');
        } catch (error) {
            console.error('Init failed:', error);
            alert('Failed to initialize. Please refresh.');
        }
    }

    setupEventListeners() {
        document.getElementById('connect-btn')?.addEventListener('click', () => {
            if (this.connected) {
                this.disconnect();
            } else {
                this.connect();
            }
        });

        document.getElementById('join-network-btn')?.addEventListener('click', () => this.joinNetwork());
        document.getElementById('refresh-networks-btn')?.addEventListener('click', () => this.refreshNetworks());
        document.getElementById('discover-btn')?.addEventListener('click', () => this.discoverPeers());
        document.getElementById('connect-all-btn')?.addEventListener('click', () => this.connectAllPeers());
        document.getElementById('sync-offline-btn')?.addEventListener('click', () => this.syncOfflineBlocks());

        document.getElementById('message-btn')?.addEventListener('click', () => this.sendMessage());
        document.getElementById('message-input')?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.sendMessage();
        });

        document.querySelector('.btn-buy')?.addEventListener('click', () => this.placeBuyOrder());
        document.querySelector('.btn-sell')?.addEventListener('click', () => this.placeSellOrder());
        document.querySelector('.refresh-btn')?.addEventListener('click', () => this.updateOrderBook());

        // GSM Roaming event listeners
        document.getElementById('deploy-roaming-contract-btn')?.addEventListener('click', () => this.deployRoamingContract());
        document.getElementById('set-rate-btn')?.addEventListener('click', () => this.setRoamingRate());
        document.getElementById('connect-roaming-btn')?.addEventListener('click', () => this.connectRoaming());
        document.getElementById('disconnect-roaming-btn')?.addEventListener('click', () => this.disconnectRoaming());
        document.getElementById('manual-billing-btn')?.addEventListener('click', () => this.processMinuteBilling());
        document.getElementById('refresh-billing-btn')?.addEventListener('click', () => this.refreshBillingHistory());
        document.getElementById('toggle-auto-billing-btn')?.addEventListener('click', () => this.toggleAutoBilling());
        
        // Wallet management event listeners
        document.getElementById('manual-transfer-btn')?.addEventListener('click', () => this.manualTransfer());
        document.getElementById('add-funds-btn')?.addEventListener('click', () => this.addFunds());
        document.getElementById('update-billing-settings-btn')?.addEventListener('click', () => this.updateBillingSettings());

        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', (e) => this.showTab(this.getTabName(e.target.textContent)));
        });
    }

    setupDefaultServer() {
        const serverInput = document.getElementById('server-input');
        if (serverInput && !serverInput.value) {
            serverInput.value = `${window.location.hostname}:3030`;
        }
    }

    getTabName(text) {
        if (text.includes('Messaging')) return 'messaging';
        if (text.includes('Trading')) return 'trading';
        if (text.includes('Order')) return 'orderbook';
        if (text.includes('Smart')) return 'contracts';
        if (text.includes('Contract')) return 'editor';
        if (text.includes('GSM') || text.includes('Roaming')) return 'roaming';
        return 'messaging';
    }

    storeBlock(block) {
        this.recentBlocks.unshift(block);
        if (this.recentBlocks.length > 10) {
            this.recentBlocks = this.recentBlocks.slice(0, 10);
        }
        console.log('Stored block #' + block.height + ', total stored:', this.recentBlocks.length);
    }

    async connect() {
        const server = document.getElementById('server-input')?.value?.trim() || `${window.location.hostname}:3030`;
        const wsUrl = `ws://${server}/ws`;
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                this.connected = true;
                console.log('Connected to tracker');
                this.refreshNetworks();
                this.updateUI();
                this.syncOfflineBlocks();
            };
            
            this.ws.onclose = () => {
                this.connected = false;
                console.log('Disconnected from tracker');
                this.updateUI();
            };
            
            this.ws.onmessage = (event) => {
                this.handleMessage(JSON.parse(event.data));
            };
            
        } catch (error) {
            console.error('Connection failed:', error);
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.connected = false;
            this.currentNetwork = null;
            this.updateUI();
        }
    }

    syncOfflineBlocks() {
        console.log('Syncing offline blocks to enterprise BC...');
        
        this.recentBlocks.forEach(block => {
            if (block.height > 0) {
                console.log('Sending offline block #' + block.height + ' to enterprise BC');
                this.send({ type: 'block', block: block });
                setTimeout(() => {}, 100);
            }
        });
        
        console.log('Finished syncing', this.recentBlocks.length, 'offline blocks');
    }

    // UPDATED: Handle all message types including enterprise sync
    handleMessage(message) {
        switch (message.type) {
            case 'network_list_update':
                this.updateNetworkDropdown(message.networks);
                break;
            case 'network_info':
                this.currentNetwork = message.network_id;
                this.updateUI();
                break;
            case 'peers':
                this.availablePeers = message.peers;
                this.connectAllPeers();
                this.updateUI();
                setTimeout(() => this.requestSync(), 2000);
                break;
            case 'offer':
                this.handleOffer(message.target, message.offer);
                break;
            case 'answer':
                this.handleAnswer(message.target, message.answer);
                break;
            case 'candidate':
                this.handleCandidate(message.target, message.candidate);
                break;
            case 'block':
                if (message.block && message.block.height) {
                    this.handleP2PBlock(message.block);
                }
                break;
            case 'enterprise_sync':
				console.log('Received enterprise sync message:', message);
				if (message.sync_data) {
					if (message.sync_data.type === 'order_book_update') {
						console.log('Processing order book update');
					   	this.handleRemoteOrderBook(message.sync_data.orders);
					} else if ( message.sync_data.type === 'trade_execution') {
						console.log('Processing trade execution from enterprise BC');
						this.handleTradeExecution(message.sync_data.trade);
					}
				}
                break;
        }
    }

    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    joinNetwork() {
        const networkId = this.getSelectedNetwork();
        if (networkId && this.connected) {
            this.send({ type: 'join_network', network_id: networkId });
            setTimeout(() => this.discoverPeers(), 1000);
        }
    }

    getSelectedNetwork() {
        const selectValue = document.getElementById('network-select')?.value;
        const inputValue = document.getElementById('network-input')?.value?.trim();
        return selectValue || inputValue;
    }
	
	handleRemoteOrderBook(orders) {
			console.log('Received remote order book update:', orders);
			console.log('Current network:', this.currentNetwork);
			
			if (orders && orders.buy_orders && orders.sell_orders) {
				// Store ALL orders from enterprise (we'll filter on display)
				this.remoteOrders = {
					bids: orders.buy_orders || [],
					asks: orders.sell_orders || []
				};
				
				console.log('All orders from enterprise:', {
					bids: this.remoteOrders.bids.length,
					asks: this.remoteOrders.asks.length
				});
				
				// Log network IDs for debugging
				this.remoteOrders.bids.forEach(bid => {
					console.log(`Bid: ${bid.quantity} @ ${bid.price} from network: ${bid.network_id}`);
				});
				this.remoteOrders.asks.forEach(ask => {
					console.log(`Ask: ${ask.quantity} @ ${ask.price} from network: ${ask.network_id}`);
				});
				
				// Force update the order book display
				this.updateOrderBook();
			}
	}


    async refreshNetworks() {
        if (!this.connected) return;
        try {
            const server = document.getElementById('server-input')?.value?.trim();
            const response = await fetch(`http://${server}/api/network-list`);
            const networks = await response.json();
            this.updateNetworkDropdown(networks);
        } catch (error) {
            console.error('Failed to refresh networks:', error);
        }
    }

    updateNetworkDropdown(networks) {
        const select = document.getElementById('network-select');
        if (!select) return;
        
        while (select.children.length > 1) {
            select.removeChild(select.lastChild);
        }
        
        networks.forEach(network => {
            const option = document.createElement('option');
            option.value = network.id;
            option.textContent = `${network.name} (${network.peer_count} peers)`;
            select.appendChild(option);
        });
    }

    discoverPeers() {
        if (this.connected) {
            this.send({ type: 'peers', peers: [] });
        }
    }

    async connectAllPeers() {
        for (const peerId of this.availablePeers) {
            if (!this.dataChannels.has(peerId)) {
                await this.connectToPeer(peerId);
            }
        }
    }

    async connectToPeer(peerId) {
        try {
            const pc = new RTCPeerConnection(this.rtcConfig);
            this.peers.set(peerId, pc);
            
            const channel = pc.createDataChannel('blockchain', { ordered: true });
            this.setupDataChannel(channel, peerId);
            
            pc.onicecandidate = (event) => {
                if (event.candidate) {
                    this.send({ type: 'candidate', target: peerId, candidate: event.candidate });
                }
            };
            
            const offer = await pc.createOffer();
            await pc.setLocalDescription(offer);
            this.send({ type: 'offer', target: peerId, offer: offer });
            
        } catch (error) {
            console.error('Failed to connect to peer:', error);
        }
    }

    setupDataChannel(channel, peerId) {
        channel.onopen = () => {
            console.log('Data channel opened for peer:', peerId.substring(0,8));
            this.dataChannels.set(peerId, channel);
            this.updateUI();
        };
        
        channel.onclose = () => {
            console.log('Data channel closed for peer:', peerId.substring(0,8));
            this.dataChannels.delete(peerId);
            this.updateUI();
        };
        
        channel.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleP2PMessage(message, peerId);
            } catch (error) {
                console.error('Failed to parse P2P message:', error);
            }
        };
    }

    async handleOffer(fromPeer, offer) {
        try {
            const pc = new RTCPeerConnection(this.rtcConfig);
            this.peers.set(fromPeer, pc);
            
            pc.ondatachannel = (event) => {
                this.setupDataChannel(event.channel, fromPeer);
            };
            
            pc.onicecandidate = (event) => {
                if (event.candidate) {
                    this.send({ type: 'candidate', target: fromPeer, candidate: event.candidate });
                }
            };
            
            await pc.setRemoteDescription(offer);
            const answer = await pc.createAnswer();
            await pc.setLocalDescription(answer);
            this.send({ type: 'answer', target: fromPeer, answer: answer });
            
        } catch (error) {
            console.error('Failed to handle offer:', error);
        }
    }

    async handleAnswer(fromPeer, answer) {
        try {
            const pc = this.peers.get(fromPeer);
            if (pc) {
                await pc.setRemoteDescription(answer);
            }
        } catch (error) {
            console.error('Failed to handle answer:', error);
        }
    }

    async handleCandidate(fromPeer, candidate) {
        try {
            const pc = this.peers.get(fromPeer);
            if (pc) {
                await pc.addIceCandidate(candidate);
            }
        } catch (error) {
            console.error('Failed to handle candidate:', error);
        }
    }

    requestSync() {
        this.broadcastToP2P({
            type: 'sync_request',
            current_height: this.blockchain.get_chain_length() - 1,
            sender: this.userId
        });
    }

    // UPDATED: Handle P2P messages including trade executions
    handleP2PMessage(message, fromPeer) {
        // Handle trade execution messages
        if (message.type === 'trade_execution' && message.sender !== this.userId) {
            console.log('Received trade execution via P2P:', message);
            this.handleTradeExecution(message.trade);
            return;
        }
        
        if (message.type === 'blockchain_block' && message.sender !== this.userId) {
            this.handleP2PBlock(message.block);
        }
        else if (message.type === 'sync_request' && message.sender !== this.userId) {
            const ourHeight = this.blockchain.get_chain_length() - 1;
            if (ourHeight > message.current_height) {
                const latestBlockJson = this.blockchain.get_latest_block_json();
                if (latestBlockJson && latestBlockJson !== '{}') {
                    const latestBlock = JSON.parse(latestBlockJson);
                    this.sendToP2PPeer(fromPeer, {
                        type: 'blockchain_block',
                        block: latestBlock,
                        sender: this.userId
                    });
                }
            }
        }
    }

    // NEW: Handle trade execution from enterprise BC
    handleTradeExecution(trade) {
		console.log('=== TRADE EXECUTION RECEIVED ===');
		console.log('Trade ID:', trade.trade_id);
		
		// Check if we've already processed this trade execution
		if (this.processedTrades.has(trade.trade_id)) {
			console.log('SKIPPING duplicate trade execution:', trade.trade_id);
			return;
		}
		
		// Mark this trade as processed
		this.processedTrades.add(trade.trade_id);
		console.log('Processing new trade execution:', trade.trade_id);
		console.log('Asset:', trade.asset);
		console.log('Quantity:', trade.quantity);
		console.log('Price:', trade.price);
		console.log('Buyer:', trade.buyer, 'Network:', trade.buyer_network);
		console.log('Seller:', trade.seller, 'Network:', trade.seller_network);


		// Update local order book if we're involved in the trade
		if (this.orderBook && 
			(trade.buyer_network === this.currentNetwork || trade.seller_network === this.currentNetwork)) {
		
			const result = this.orderBook.execute_cross_network_trade(
						trade.asset,
						trade.quantity,
						trade.price,
						trade.buyer,
						trade.seller
					);
		
			console.log('Local order book update result:', result);
		}
	    
		
		// Create a trade execution transaction
		const executionTx = {
		    id: `exec_${trade.trade_id}`,
		    from: trade.buyer,
		    to: trade.seller,
		    amount: Math.floor(trade.quantity * trade.price / 100),
		    tx_type: {
			TradeExecution: {
			    asset: trade.asset,
			    quantity: trade.quantity,
			    price: trade.price,
			    buyer: trade.buyer,
			    seller: trade.seller,
			    trade_id: trade.trade_id
			}
		    },
		    timestamp: trade.timestamp
		};
		
		// Add to blockchain
		this.blockchain.add_p2p_transaction(JSON.stringify(executionTx));
		const success = this.blockchain.mine_block();
		
		if (success) {
		    const execBlock = JSON.parse(this.blockchain.get_latest_block_json());
		    this.storeBlock(execBlock);
		    
		    console.log('Created trade execution block #' + execBlock.height);
		    
		    // Broadcast to other peers in this network
		    this.broadcastToP2P({
			type: 'blockchain_block',
			block: execBlock,
			sender: this.userId
		    });
		    
		    // Send to enterprise BC if connected
		    if (this.connected && execBlock.height > 0) {
			this.send({ type: 'block', block: execBlock });
		    }
		    
		    console.log('Trade execution processed and broadcast');
		}
		
		// Update UI to show the trade
		this.updateOrderBook();
		this.updateUI();
		this.showTradeNotification(trade);
	    }

	
	// Add method to cancel an order
	cancelOrder(orderId) {
	    if (this.orderBook) {
		const success = this.orderBook.cancel_order(orderId);
		if (success) {
		    console.log('Order cancelled:', orderId);
		    this.updateOrderBook();

		    // Notify enterprise BC about cancellation
		    if (this.connected) {
			// could send a cancellation message to enterprise
			// this.send({ type: 'cancel_order', order_id: orderId });
		    }
		}
	    }
	}

	// Add this new method after handleTradeExecution
	updateLocalOrderBookAfterTrade(trade) {
		// Get current order book
		const orderBookData = JSON.parse(this.orderBook.get_order_book_json());
			
		// Check if we need to update our local orders
		if (trade.buyer_network === this.currentNetwork || trade.seller_network === this.currentNetwork) {
			console.log('Updating local order book after trade execution');
			
			// If we're the buyer, remove/reduce from our buy orders
			if (trade.buyer_network === this.currentNetwork) {
				// Remove the executed quantity from local buy orders
				// Note: This is handled by the WASM orderbook when we add the trade
			}
				
			// If we're the seller, remove/reduce from our sell orders  
			if (trade.seller_network === this.currentNetwork) {
				// Remove the executed quantity from local sell orders
				// Note: This is handled by the WASM orderbook when we add the trade
			}
				
			// Force a full refresh of the order book
			this.updateOrderBook();
		}
	}



    // NEW: Show trade notification
    showTradeNotification(trade) {
        const quantity = (trade.quantity / 100).toFixed(2);
        const price = (trade.price / 100).toFixed(2);
        const message = `TRADE EXECUTED: ${quantity} ${trade.asset} @ $${price}`;
        
        // Create a temporary notification element
        const notification = document.createElement('div');
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #4CAF50;
            color: white;
            padding: 15px;
            border-radius: 5px;
            z-index: 1000;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
            max-width: 300px;
        `;
        notification.innerHTML = `
            <strong>Cross-Network Trade</strong><br>
            ${message}<br>
            <small>Between networks ${trade.buyer_network} and ${trade.seller_network}</small>
        `;
        
        document.body.appendChild(notification);
        
        // Remove after 5 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 5000);
    }

    handleP2PBlock(block) {
        if (!block || !block.height) return;
        
        try {
            const currentHeight = this.blockchain.get_chain_length() - 1;
            
            if (block.height > currentHeight) {
                if (block.validator && block.stake_weight) {
                    try {
                        this.blockchain.add_validator(block.validator, block.stake_weight);
                    } catch (e) {
                        // Validator already exists
                    }
                }
                
                const success = this.blockchain.add_p2p_block(JSON.stringify(block));
                if (success) {
                    this.storeBlock(block);
                    
                    // Update OrderBook if trading transaction
                    if (block.transactions) {
                        for (const tx of block.transactions) {
                            if (tx.tx_type && tx.tx_type.Trading) {
                                const trading = tx.tx_type.Trading;
                                if (tx.id.includes('buy_')) {
                                    this.orderBook.place_buy_order(tx.from, trading.asset, trading.quantity, trading.price);
                                } else if (tx.id.includes('sell_')) {
                                    this.orderBook.place_sell_order(tx.from, trading.asset, trading.quantity, trading.price);
                                }
                            }
                        }
                        this.updateOrderBook();
                    }
                    
                    this.updateUI();
                }
            }
        } catch (error) {
            console.error('Error processing P2P block:', error);
        }
    }

    sendToP2PPeer(peerId, message) {
        const channel = this.dataChannels.get(peerId);
        if (channel && channel.readyState === 'open') {
            try {
                channel.send(JSON.stringify(message));
                return true;
            } catch (error) {
                console.error('Failed to send to peer:', error);
            }
        }
        return false;
    }

    broadcastToP2P(message) {
        let sent = 0;
        for (const [peerId, channel] of this.dataChannels) {
            if (channel.readyState === 'open') {
                try {
                    channel.send(JSON.stringify(message));
                    sent++;
                } catch (error) {
                    console.error('Failed to send to peer:', peerId.substring(0,8), error);
                }
            }
        }
        return sent;
    }

    sendMessage() {
        const input = document.getElementById('message-input');
        const messageText = input?.value?.trim();
        if (!messageText) return;

        try {
            this.blockchain.add_message(messageText, this.userId);
            const success = this.blockchain.mine_block();
            
            if (success) {
                const latestBlockJson = this.blockchain.get_latest_block_json();
                if (latestBlockJson && latestBlockJson !== '{}') {
                    const minedBlock = JSON.parse(latestBlockJson);
                    this.storeBlock(minedBlock);
                    
                    this.broadcastToP2P({
                        type: 'blockchain_block',
                        block: minedBlock,
                        sender: this.userId
                    });
                    
                    // Only send non-genesis blocks to enterprise
                    if (this.connected && minedBlock.height > 0) {
                        this.send({ type: 'block', block: minedBlock });
                    }
                }
            }
            
            input.value = '';
            this.updateUI();
            
        } catch (error) {
            console.error('Error sending message:', error);
        }
    }

	placeBuyOrder() {
    const asset = document.getElementById('buy-asset')?.value;
    const quantity = parseFloat(document.getElementById('buy-quantity')?.value) || 0;
    const price = parseFloat(document.getElementById('buy-price')?.value) || 0;

    if (!asset || quantity <= 0 || price <= 0) return;

    try {
        const quantityInt = Math.floor(quantity * 100);
        const priceInt = Math.floor(price * 100);

        // Step 1: Create BUY order block
        this.blockchain.call_contract_buy(asset, quantityInt, priceInt, this.userId);
        const orderSuccess = this.blockchain.mine_block();

        if (orderSuccess) {
            const orderBlock = JSON.parse(this.blockchain.get_latest_block_json());
            this.storeBlock(orderBlock);
            this.broadcastToP2P({ type: 'blockchain_block', block: orderBlock, sender: this.userId });
            if (this.connected && orderBlock.height > 0) {
                this.send({ type: 'block', block: orderBlock });
            }
        }
        
        // Step 2: Place order ONCE in local order book
        const orderId = this.orderBook.place_buy_order(this.userId, asset, quantityInt, priceInt);
        console.log('Placed buy order locally with ID:', orderId);
        
        // Rest of the execution logic remains the same...
        const recentTrades = JSON.parse(this.orderBook.get_recent_trades_json());
        const currentTime = Math.floor(Date.now() / 1000);
        
        const newTrade = recentTrades.find(trade => 
            Math.abs(trade.timestamp - currentTime) < 2 &&
            trade.buyer === this.userId &&
            trade.asset === asset
        );
        
        if (newTrade) {
            const executionTx = {
                id: `exec_${Date.now()}`,
                from: newTrade.buyer,
                to: newTrade.seller,
                amount: Math.floor(newTrade.quantity * newTrade.price / 100),
                tx_type: {
                    TradeExecution: {
                        asset: newTrade.asset,
                        quantity: newTrade.quantity,
                        price: newTrade.price,
                        buyer: newTrade.buyer,
                        seller: newTrade.seller
                    }
                },
                timestamp: currentTime
            };
            
            this.blockchain.add_p2p_transaction(JSON.stringify(executionTx));
            const execSuccess = this.blockchain.mine_block();
            
            if (execSuccess) {
                const execBlock = JSON.parse(this.blockchain.get_latest_block_json());
                this.storeBlock(execBlock);
                this.broadcastToP2P({ type: 'blockchain_block', block: execBlock, sender: this.userId });
                if (this.connected && execBlock.height > 0) {
                    this.send({ type: 'block', block: execBlock });
                }
            }
        }

        this.clearBuyForm();
        
        // Only update order book once
        setTimeout(() => this.updateOrderBook(), 100);
        this.updateUI();
        
    } catch (error) {
        console.error('Error placing buy order:', error);
    }
}

// Similar update for placeSellOrder
placeSellOrder() {
    const asset = document.getElementById('sell-asset')?.value;
    const quantity = parseFloat(document.getElementById('sell-quantity')?.value) || 0;
    const price = parseFloat(document.getElementById('sell-price')?.value) || 0;

    if (!asset || quantity <= 0 || price <= 0) return;

    try {
        const quantityInt = Math.floor(quantity * 100);
        const priceInt = Math.floor(price * 100);

        // Step 1: Create SELL order block
        this.blockchain.call_contract_sell(asset, quantityInt, priceInt, this.userId);
        const orderSuccess = this.blockchain.mine_block();

        if (orderSuccess) {
            const orderBlock = JSON.parse(this.blockchain.get_latest_block_json());
            this.storeBlock(orderBlock);
            this.broadcastToP2P({ type: 'blockchain_block', block: orderBlock, sender: this.userId });
            if (this.connected && orderBlock.height > 0) {
                this.send({ type: 'block', block: orderBlock });
            }
        }
        
        // Step 2: Place order ONCE in local order book
        const orderId = this.orderBook.place_sell_order(this.userId, asset, quantityInt, priceInt);
        console.log('Placed sell order locally with ID:', orderId);
        
        // Rest of the execution logic remains the same...
        const recentTrades = JSON.parse(this.orderBook.get_recent_trades_json());
        const currentTime = Math.floor(Date.now() / 1000);
        
        const newTrade = recentTrades.find(trade => 
            Math.abs(trade.timestamp - currentTime) < 2 &&
            trade.seller === this.userId &&
            trade.asset === asset
        );
        
        if (newTrade) {
            const executionTx = {
                id: `exec_${Date.now()}`,
                from: newTrade.buyer,
                to: newTrade.seller,
                amount: Math.floor(newTrade.quantity * newTrade.price / 100),
                tx_type: {
                    TradeExecution: {
                        asset: newTrade.asset,
                        quantity: newTrade.quantity,
                        price: newTrade.price,
                        buyer: newTrade.buyer,
                        seller: newTrade.seller
                    }
                },
                timestamp: currentTime
            };
            
            this.blockchain.add_p2p_transaction(JSON.stringify(executionTx));
            const execSuccess = this.blockchain.mine_block();
            
            if (execSuccess) {
                const execBlock = JSON.parse(this.blockchain.get_latest_block_json());
                this.storeBlock(execBlock);
                this.broadcastToP2P({ type: 'blockchain_block', block: execBlock, sender: this.userId });
                if (this.connected && execBlock.height > 0) {
                    this.send({ type: 'block', block: execBlock });
                }
            }
        }

        this.clearSellForm();
        
        // Only update order book once
        setTimeout(() => this.updateOrderBook(), 100);
        this.updateUI();
        
    } catch (error) {
        console.error('Error placing sell order:', error);
    }
}

    clearBuyForm() {
        const qty = document.getElementById('buy-quantity');
        const price = document.getElementById('buy-price');
        if (qty) qty.value = '';
        if (price) price.value = '';
    }

    clearSellForm() {
        const qty = document.getElementById('sell-quantity');
        const price = document.getElementById('sell-price');
        if (qty) qty.value = '';
        if (price) price.value = '';
    }

	updateOrderBook() {
		if (!this.orderBook) return;

		try {
			// Get local order book only
			const localOrderBook = JSON.parse(this.orderBook.get_order_book_json());
			const tradesData = JSON.parse(this.orderBook.get_recent_trades_json());

			// Update tables with local orders (remote will be added in updateTable)
			this.updateTable('bids-tbody', localOrderBook.bids || [], 'bid');
			this.updateTable('asks-tbody', localOrderBook.asks || [], 'ask');
			this.updateTradesTable(tradesData || []);

		} catch (error) {
			console.error('Error updating order book:', error);
		}
	}

// Keep the hybrid updateTable for online/offline support
//

updateTable(tableId, localOrders, type) {
    const tbody = document.getElementById(tableId);
    if (!tbody) return;
    
    let allOrders = [];
    
    // Online: use enterprise data | Offline: use local data only
    if (this.connected && this.remoteOrders && this.remoteOrders.bids && this.remoteOrders.asks) {
        const enterpriseOrders = type === 'bid' ? this.remoteOrders.bids : this.remoteOrders.asks;
        allOrders = enterpriseOrders.map(order => ({
            ...order,
            isLocal: order.network_id === this.currentNetwork
        }));
    } else {
        allOrders = localOrders.map(order => ({
            ...order,
            network_id: this.currentNetwork || 'local',
            isLocal: true
        }));
    }
    
    if (allOrders.length === 0) {
        tbody.innerHTML = `<tr><td colspan="4">No ${type} orders</td></tr>`;
        return;
    }
    
    allOrders.sort((a, b) => {
        const priceA = a.price || 0;
        const priceB = b.price || 0;
        return type === 'bid' ? priceB - priceA : priceA - priceB;
    });
    
    tbody.innerHTML = allOrders.map(order => {
        const network = order.network_id || 'local';
        const traderDisplay = (order.trader || 'unknown').substring(0, 8);
        const label = this.connected ? `(${network})` : '(offline)';
        
        return `
            <tr>
                <td>$${((order.price || 0) / 100).toFixed(2)}</td>
                <td>${((order.quantity || 0) / 100).toFixed(2)}</td>
                <td>${order.asset || 'N/A'}</td>
                <td>${traderDisplay}... ${label}</td>
            </tr>
        `;
    }).join('');
}


    updateTradesTable(trades) {
        const tbody = document.getElementById('trades-tbody');
        if (!tbody) return;

        if (trades.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6">No trades yet</td></tr>';
            return;
        }

        tbody.innerHTML = trades.map(trade => `
            <tr>
                <td>${new Date(trade.timestamp * 1000).toLocaleTimeString()}</td>
                <td>${trade.asset}</td>
                <td>${(trade.quantity / 100).toFixed(2)}</td>
                <td>$${(trade.price / 100).toFixed(2)}</td>
                <td>${trade.buyer.substring(0, 8)}...</td>
                <td>${trade.seller.substring(0, 8)}...</td>
            </tr>
        `).join('');
    }

    updateUI() {
        this.updateConnectionStatus();
        this.updateButtonStates();
        this.updateBlockchainDisplay();
    }

    updateConnectionStatus() {
        const peerCount = document.getElementById('peer-count');
        const blockCount = document.getElementById('block-count');
        const networkName = document.getElementById('network-name');
        const status = document.getElementById('status');

        if (peerCount) peerCount.textContent = this.dataChannels.size;
        if (blockCount && this.blockchain) blockCount.textContent = this.blockchain.get_chain_length();
        if (networkName) networkName.textContent = this.currentNetwork || 'None';
        if (status) status.textContent = this.connected ? 'Connected' : 'Offline';
    }

    updateButtonStates() {
        const connectBtn = document.getElementById('connect-btn');
        const joinBtn = document.getElementById('join-network-btn');
        const discoverBtn = document.getElementById('discover-btn');
        const connectAllBtn = document.getElementById('connect-all-btn');
        const messageBtn = document.getElementById('message-btn');
        const syncBtn = document.getElementById('sync-offline-btn');

        if (connectBtn) {
            connectBtn.textContent = this.connected ? 'Disconnect' : 'Connect';
        }
        
        if (joinBtn) joinBtn.disabled = !this.connected;
        if (discoverBtn) discoverBtn.disabled = !this.currentNetwork;
        if (connectAllBtn) connectAllBtn.disabled = !this.currentNetwork || !this.availablePeers.length;
        if (messageBtn) messageBtn.disabled = !this.connected;
        if (syncBtn) syncBtn.disabled = !this.connected || this.recentBlocks.length === 0;
    }

    updateBlockchainDisplay() {
        if (!this.blockchain) return;

        const displays = ['blockchain', 'blockchain-trading', 'blockchain-orderbook', 'blockchain-contracts', 'blockchain-editor'];
        
        displays.forEach(id => {
            const div = document.getElementById(id);
            if (div) {
                this.updateBlockchainElement(div);
            }
        });
    }

    updateBlockchainElement(div) {
        try {
            const chainLength = this.blockchain.get_chain_length();
            const pendingCount = this.blockchain.get_pending_count();
            
            let html = `
                <h4>Blockchain Status</h4>
                <p><strong>Blocks:</strong> ${chainLength - 1} | <strong>Pending:</strong> ${pendingCount}</p>
            `;
            
            if (this.recentBlocks.length > 0) {
                this.recentBlocks.forEach(block => {
                    html += this.generateBlockDisplay(block);
                });
            } else {
                html += '<p>No blocks yet - send a message or place an order</p>';
            }
            
            div.innerHTML = html;
            
        } catch (error) {
            div.innerHTML = `<p>Error loading blockchain: ${error.message}</p>`;
        }
    }

    // UPDATED: Generate block display with trade execution support
    generateBlockDisplay(block) {
        if (!block || !block.transactions || block.transactions.length === 0) return '';

        const tx = block.transactions[0];
        let content = '';
        let typeLabel = 'Transaction';
        
        if (tx.tx_type?.Message) {
            content = tx.tx_type.Message.content;
            typeLabel = 'Message';
        } else if (tx.tx_type?.Trading) {
            const trading = tx.tx_type.Trading;
            const quantity = (trading.quantity / 100).toFixed(2);
            const price = (trading.price / 100).toFixed(2);
            
            const orderType = tx.id.includes('buy_') ? "BUY" : "SELL";
            content = `${orderType} ORDER: ${quantity} ${trading.asset} @ $${price}`;
            typeLabel = `${orderType} Order`;
        } else if (tx.tx_type?.TradeExecution) {
            const trade = tx.tx_type.TradeExecution;
            const quantity = (trade.quantity / 100).toFixed(2);
            const price = (trade.price / 100).toFixed(2);
            content = `CROSS-NETWORK TRADE: ${quantity} ${trade.asset} @ $${price}`;
            typeLabel = 'Trade Execution';
        } else {
            content = `Transaction ID: ${tx.id}`;
            typeLabel = 'Transaction';
        }

        return `
            <div class="block-display">
                <div style="display: flex; justify-content: space-between; margin-bottom: 10px;">
                    <span><strong>${typeLabel}</strong> - Block #${block.height}</span>
                    <span>${new Date(block.timestamp * 1000).toLocaleTimeString()}</span>
                </div>
                <div style="background: #f8f9fa; padding: 10px; border-radius: 4px;">
                    "${content}"
                </div>
                <div style="font-size: 12px; color: #666; margin-top: 10px;">
                    From: ${tx.from?.substring(0, 12)}... | Network: ${this.currentNetwork || 'Local'}
                </div>
            </div>
        `;
    }

    showTab(tabName) {
        document.querySelectorAll('.tab-panel').forEach(panel => {
            panel.classList.remove('active');
        });
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        
        const selectedPanel = document.getElementById(tabName);
        if (selectedPanel) selectedPanel.classList.add('active');
        
        document.querySelectorAll('.tab').forEach(tab => {
            const tabText = tab.textContent.trim();
            if ((tabName === 'messaging' && tabText.includes('Messaging')) ||
                (tabName === 'trading' && tabText.includes('Trading')) ||
                (tabName === 'orderbook' && tabText.includes('Order')) ||
                (tabName === 'contracts' && tabText.includes('Smart')) ||
                (tabName === 'editor' && tabText.includes('Contract'))) {
                tab.classList.add('active');
            }
        });
        
        this.updateUI();
    }

    // Wallet Management
    initializeWallets() {
        // Initialize default wallets with starting balances
        this.wallets.set('wallet_vodafone', { 
            balance: 1000, 
            network: 'Vodafone',
            transactions: []
        });
        this.wallets.set('wallet_tmobile', { 
            balance: 0, 
            network: 'T-Mobile',
            transactions: []
        });
        this.wallets.set('wallet_orange', { 
            balance: 800, 
            network: 'Orange',
            transactions: []
        });
        this.wallets.set('wallet_verizon', { 
            balance: 0, 
            network: 'Verizon',
            transactions: []
        });
    }

    getWalletBalance(walletId) {
        const wallet = this.wallets.get(walletId);
        return wallet ? wallet.balance : 0;
    }

    transferFunds(fromWallet, toWallet, amount, description = '') {
        const from = this.wallets.get(fromWallet);
        const to = this.wallets.get(toWallet);

        if (!from || !to) {
            console.error('Invalid wallet addresses');
            return false;
        }

        if (from.balance < amount) {
            console.error(`Insufficient funds: ${from.balance} < ${amount}`);
            this.logRoamingEvent(`❌ Transfer failed: Insufficient funds in ${fromWallet}`, 'error');
            return false;
        }

        // Execute transfer
        from.balance -= amount;
        to.balance += amount;

        // Log transaction
        const transaction = {
            timestamp: Date.now(),
            from: fromWallet,
            to: toWallet,
            amount,
            description,
            id: `tx_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`
        };

        from.transactions.push({...transaction, type: 'debit'});
        to.transactions.push({...transaction, type: 'credit'});

        console.log(`💸 Transfer: ${fromWallet} → ${toWallet}: ${amount} units`);
        this.logRoamingEvent(`💸 ${description || 'Transfer'}: ${amount} units (${fromWallet} → ${toWallet})`, 'success');
        
        this.updateWalletDisplay();
        return true;
    }

    updateWalletDisplay() {
        // Update wallet display in GSM Roaming tab
        const walletContainer = document.getElementById('wallet-status');
        if (!walletContainer) return;

        const walletsHtml = Array.from(this.wallets.entries()).map(([id, wallet]) => {
            const recentTx = wallet.transactions.slice(-3).reverse();
            const txHtml = recentTx.map(tx => {
                const sign = tx.type === 'credit' ? '+' : '-';
                const color = tx.type === 'credit' ? '#28a745' : '#dc3545';
                return `<div style="color: ${color}; font-size: 12px;">${sign}${tx.amount} - ${tx.description}</div>`;
            }).join('');

            return `
                <div class="wallet-card">
                    <h4>💼 ${id}</h4>
                    <div class="wallet-info">
                        <div><strong>Network:</strong> ${wallet.network}</div>
                        <div><strong>Balance:</strong> ${wallet.balance} units</div>
                        <div><strong>Transactions:</strong> ${wallet.transactions.length}</div>
                    </div>
                    ${txHtml ? `<div class="recent-tx">${txHtml}</div>` : ''}
                </div>
            `;
        }).join('');

        walletContainer.innerHTML = walletsHtml;
    }

    // GSM Roaming Methods
    deployRoamingContract() {
        try {
            console.log('🚀 Deploying GSM Roaming Smart Contract...');
            this.logRoamingEvent('Deploying GSM Roaming Contract...', 'success');
            
            // In a real implementation, this would deploy the contract via the blockchain
            setTimeout(() => {
                this.logRoamingEvent('GSM Roaming Contract deployed successfully!', 'success');
            }, 1000);
        } catch (error) {
            console.error('Error deploying roaming contract:', error);
            this.logRoamingEvent('Failed to deploy roaming contract', 'error');
        }
    }

    setRoamingRate() {
        try {
            const homeNetwork = document.getElementById('home-network')?.value;
            const visitingNetwork = document.getElementById('visiting-network')?.value;
            const rate = document.getElementById('roaming-rate')?.value;

            if (!homeNetwork || !visitingNetwork || !rate) {
                alert('Please fill in all network rate fields');
                return;
            }

            console.log(`💰 Setting rate: ${homeNetwork} -> ${visitingNetwork} = ${rate} units/min`);
            this.logRoamingEvent(`Rate set: ${homeNetwork} → ${visitingNetwork} at ${rate} units/minute`, 'success');
            
        } catch (error) {
            console.error('Error setting roaming rate:', error);
            this.logRoamingEvent('Failed to set roaming rate', 'error');
        }
    }

    connectRoaming() {
        try {
            const imsi = document.getElementById('device-imsi')?.value;
            const phoneNumber = document.getElementById('phone-number')?.value || '+1-555-0000';
            const homeNetwork = document.getElementById('home-network')?.value;
            const visitingNetwork = document.getElementById('visiting-network')?.value;

            if (!imsi || !homeNetwork || !visitingNetwork) {
                alert('Please fill in IMSI and network fields');
                return;
            }

            // Check if actually roaming (different networks)
            const isRoaming = homeNetwork !== visitingNetwork;
            if (!isRoaming) {
                this.logRoamingEvent(`📱 Connected to home network ${homeNetwork} - No roaming charges`, 'success');
                return;
            }

            console.log(`📡 Roaming: ${phoneNumber} (${imsi}) from ${homeNetwork} → ${visitingNetwork}`);
            
            // Get billing rate from either roaming-rate field or billing-units field
            const roamingRate = parseInt(document.getElementById('roaming-rate')?.value || '15');
            const billingUnits = parseInt(document.getElementById('billing-units')?.value || roamingRate.toString());
            
            // Create session - representing the roaming agreement activation
            const sessionId = `${imsi}_${Date.now()}`;
            const session = {
                sessionId,
                imsi,
                phoneNumber,
                homeNetwork,
                visitingNetwork,
                roamingAgreement: `${homeNetwork}-${visitingNetwork}`,
                startTime: Date.now(),
                minutesBilled: 0,
                totalCost: 0,
                rate: billingUnits,
                status: 'ROAMING_ACTIVE'
            };

            // Store session (in real implementation, this would be in blockchain)
            if (!this.roamingSessions) this.roamingSessions = new Map();
            this.roamingSessions.set(sessionId, session);
            this.currentRoamingSession = sessionId;

            // Update UI
            this.updateActiveSessionsDisplay();
            this.logRoamingEvent(`🌍 ROAMING ACTIVATED: ${phoneNumber} (${homeNetwork} → ${visitingNetwork}) - Agreement: ${homeNetwork}-${visitingNetwork}`, 'success');
            this.logRoamingEvent(`💰 Billing Rate: ${billingUnits} units per interval - Based on inter-operator agreement`, 'warning');
            
            // Enable buttons
            document.getElementById('disconnect-roaming-btn').disabled = false;
            document.getElementById('manual-billing-btn').disabled = false;
            document.getElementById('connect-roaming-btn').disabled = true;

        } catch (error) {
            console.error('Error connecting to roaming network:', error);
            this.logRoamingEvent('Failed to connect to roaming network', 'error');
        }
    }

    disconnectRoaming() {
        try {
            if (!this.currentRoamingSession) {
                alert('No active roaming session');
                return;
            }

            const session = this.roamingSessions.get(this.currentRoamingSession);
            if (!session) return;

            // Calculate final cost
            const durationMinutes = Math.max(1, Math.floor((Date.now() - session.startTime) / 60000));
            const finalCost = durationMinutes * session.rate;

            console.log(`📴 Disconnecting session ${this.currentRoamingSession}`);
            this.logRoamingEvent(`📴 Session ended. Duration: ${durationMinutes}min, Cost: ${finalCost} units`, 'warning');

            // Remove from active sessions
            this.roamingSessions.delete(this.currentRoamingSession);
            this.currentRoamingSession = null;

            // Update UI
            this.updateActiveSessionsDisplay();
            
            // Reset buttons
            document.getElementById('disconnect-roaming-btn').disabled = true;
            document.getElementById('manual-billing-btn').disabled = true;
            document.getElementById('connect-roaming-btn').disabled = false;

        } catch (error) {
            console.error('Error disconnecting from roaming network:', error);
            this.logRoamingEvent('Failed to disconnect from roaming network', 'error');
        }
    }

    processMinuteBilling() {
        try {
            if (!this.currentRoamingSession) {
                alert('No active roaming session');
                return;
            }

            const session = this.roamingSessions.get(this.currentRoamingSession);
            if (!session) return;

            const guestWallet = document.getElementById('guest-wallet')?.value || 'wallet_vodafone';
            const hostWallet = document.getElementById('host-wallet')?.value || 'wallet_tmobile';

            // Attempt to transfer funds based on roaming agreement
            const transferSuccess = this.transferFunds(
                guestWallet, 
                hostWallet, 
                session.rate, 
                `Roaming Agreement ${session.roamingAgreement} - Interval ${session.minutesBilled + 1}`
            );

            if (!transferSuccess) {
                this.logRoamingEvent('⚠️ Billing failed: Insufficient funds - session suspended', 'error');
                return;
            }

            session.minutesBilled += 1;
            session.totalCost += session.rate;

            console.log(`💳 Billing minute ${session.minutesBilled}: ${session.rate} units`);
            this.logRoamingEvent(`💳 Minute ${session.minutesBilled} billed: ${session.rate} units (Total: ${session.totalCost})`, 'warning');

            // Update session in map
            this.roamingSessions.set(this.currentRoamingSession, session);
            this.updateActiveSessionsDisplay();

        } catch (error) {
            console.error('Error processing minute billing:', error);
            this.logRoamingEvent('Failed to process minute billing', 'error');
        }
    }

    refreshBillingHistory() {
        try {
            console.log('🔄 Refreshing billing history...');
            this.logRoamingEvent('Billing history refreshed', 'success');
            // In real implementation, this would fetch from blockchain
        } catch (error) {
            console.error('Error refreshing billing history:', error);
        }
    }

    toggleAutoBilling() {
        try {
            const button = document.getElementById('toggle-auto-billing-btn');
            const statusSpan = document.getElementById('billing-status');
            const intervalSpan = document.getElementById('current-interval');
            const rateSpan = document.getElementById('current-rate');
            
            // Get custom interval and units
            const intervalSeconds = parseInt(document.getElementById('billing-interval')?.value || '60');
            const billingUnits = parseInt(document.getElementById('billing-units')?.value || '15');

            if (!this.autoBillingInterval) {
                // Update rate for current session if exists
                if (this.currentRoamingSession && this.roamingSessions) {
                    const session = this.roamingSessions.get(this.currentRoamingSession);
                    if (session) {
                        session.rate = billingUnits;
                        this.roamingSessions.set(this.currentRoamingSession, session);
                    }
                }

                // Start auto billing with custom interval
                this.autoBillingInterval = setInterval(() => {
                    if (this.currentRoamingSession) {
                        this.processMinuteBilling();
                    }
                }, intervalSeconds * 1000); // Convert seconds to milliseconds

                // Store settings for display
                this.billingIntervalSeconds = intervalSeconds;
                this.billingUnitsAmount = billingUnits;

                button.textContent = '⏸️ Stop Auto Billing';
                statusSpan.textContent = 'Enabled';
                statusSpan.style.color = '#28a745';
                intervalSpan.textContent = intervalSeconds;
                rateSpan.textContent = billingUnits;
                
                this.logRoamingEvent(`⏰ Automatic billing started (every ${intervalSeconds} seconds, ${billingUnits} units per interval)`, 'success');
            } else {
                // Stop auto billing
                clearInterval(this.autoBillingInterval);
                this.autoBillingInterval = null;

                button.textContent = '▶️ Start Auto Billing';
                statusSpan.textContent = 'Disabled';
                statusSpan.style.color = '#dc3545';
                this.logRoamingEvent('⏸️ Automatic billing stopped', 'warning');
            }
        } catch (error) {
            console.error('Error toggling auto billing:', error);
            this.logRoamingEvent('Failed to toggle auto billing', 'error');
        }
    }

    updateActiveSessionsDisplay() {
        const container = document.getElementById('active-sessions');
        if (!container) return;

        if (!this.roamingSessions || this.roamingSessions.size === 0) {
            container.innerHTML = '<div class="info-banner">No active roaming sessions</div>';
            return;
        }

        const sessionsHtml = Array.from(this.roamingSessions.values()).map(session => {
            const duration = Math.floor((Date.now() - session.startTime) / 1000);
            const minutes = Math.floor(duration / 60);
            const seconds = duration % 60;

            return `
                <div class="active-session">
                    <h4>🌍 ROAMING: ${session.phoneNumber || session.imsi}</h4>
                    <div class="session-details">
                        <div class="session-detail"><strong>Status:</strong> <span style="color: #28a745">● ${session.status}</span></div>
                        <div class="session-detail"><strong>Agreement:</strong> ${session.roamingAgreement}</div>
                        <div class="session-detail"><strong>Duration:</strong> ${minutes}:${seconds.toString().padStart(2, '0')}</div>
                        <div class="session-detail"><strong>Intervals Billed:</strong> ${session.minutesBilled}</div>
                        <div class="session-detail"><strong>Rate:</strong> ${session.rate} units/interval</div>
                        <div class="session-detail"><strong>Total Cost:</strong> ${session.totalCost} units</div>
                        <div class="session-detail"><strong>Home Provider:</strong> ${session.homeNetwork}</div>
                        <div class="session-detail"><strong>Visiting Network:</strong> ${session.visitingNetwork}</div>
                    </div>
                </div>
            `;
        }).join('');

        container.innerHTML = sessionsHtml;
    }

    logRoamingEvent(message, type = 'info') {
        const container = document.getElementById('roaming-events');
        if (!container) return;

        const timestamp = new Date().toLocaleTimeString();
        const eventDiv = document.createElement('div');
        eventDiv.className = `roaming-event ${type}`;
        eventDiv.innerHTML = `
            ${message}
            <span class="timestamp">${timestamp}</span>
        `;

        container.insertBefore(eventDiv, container.firstChild);

        // Keep only last 20 events
        while (container.children.length > 20) {
            container.removeChild(container.lastChild);
        }
    }

    // Manual wallet management methods
    manualTransfer() {
        try {
            const fromWallet = document.getElementById('transfer-from')?.value;
            const toWallet = document.getElementById('transfer-to')?.value;
            const amount = parseInt(document.getElementById('transfer-amount')?.value || '0');

            if (!fromWallet || !toWallet || amount <= 0) {
                alert('Please select valid wallets and amount');
                return;
            }

            if (fromWallet === toWallet) {
                alert('Cannot transfer to the same wallet');
                return;
            }

            const success = this.transferFunds(fromWallet, toWallet, amount, 'Manual transfer');
            if (success) {
                this.logRoamingEvent(`✅ Manual transfer completed: ${amount} units`, 'success');
            }
        } catch (error) {
            console.error('Error in manual transfer:', error);
            this.logRoamingEvent('❌ Manual transfer failed', 'error');
        }
    }

    addFunds() {
        try {
            const vodafoneWallet = this.wallets.get('wallet_vodafone');
            if (vodafoneWallet) {
                vodafoneWallet.balance += 500;
                vodafoneWallet.transactions.push({
                    timestamp: Date.now(),
                    from: 'system',
                    to: 'wallet_vodafone',
                    amount: 500,
                    description: 'System fund addition',
                    type: 'credit',
                    id: `sys_${Date.now()}`
                });
                
                this.updateWalletDisplay();
                this.logRoamingEvent('💰 Added 500 units to wallet_vodafone', 'success');
            }
        } catch (error) {
            console.error('Error adding funds:', error);
            this.logRoamingEvent('❌ Failed to add funds', 'error');
        }
    }

    updateBillingSettings() {
        try {
            const intervalSeconds = parseInt(document.getElementById('billing-interval')?.value || '60');
            const billingUnits = parseInt(document.getElementById('billing-units')?.value || '15');
            
            // Update current session if exists
            if (this.currentRoamingSession && this.roamingSessions) {
                const session = this.roamingSessions.get(this.currentRoamingSession);
                if (session) {
                    session.rate = billingUnits;
                    this.roamingSessions.set(this.currentRoamingSession, session);
                    this.updateActiveSessionsDisplay();
                }
            }

            // If auto-billing is active, restart with new interval
            if (this.autoBillingInterval) {
                clearInterval(this.autoBillingInterval);
                
                this.autoBillingInterval = setInterval(() => {
                    if (this.currentRoamingSession) {
                        this.processMinuteBilling();
                    }
                }, intervalSeconds * 1000);

                // Update display
                document.getElementById('current-interval').textContent = intervalSeconds;
                document.getElementById('current-rate').textContent = billingUnits;
                
                this.logRoamingEvent(`⚙️ Billing settings updated: ${intervalSeconds}s interval, ${billingUnits} units/interval`, 'success');
            } else {
                // Just update the display for next time
                document.getElementById('current-interval').textContent = intervalSeconds;
                document.getElementById('current-rate').textContent = billingUnits;
                this.logRoamingEvent(`⚙️ Settings saved for next billing session: ${intervalSeconds}s, ${billingUnits} units`, 'success');
            }

        } catch (error) {
            console.error('Error updating billing settings:', error);
            this.logRoamingEvent('❌ Failed to update billing settings', 'error');
        }
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    try {
        const app = new DistliApp();
        await app.init();
        window.app = app;
        console.log('App ready for cross-network trading');
    } catch (error) {
        console.error('Failed to start app:', error);
        alert('Failed to start application. Please refresh.');
    }
});

window.sendMessage = () => window.app?.sendMessage();
window.placeBuyOrder = () => window.app?.placeBuyOrder();
window.placeSellOrder = () => window.app?.placeSellOrder();
window.updateOrderBook = () => window.app?.updateOrderBook();

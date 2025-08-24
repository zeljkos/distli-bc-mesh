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
        
        this.rtcConfig = {
            iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
        };
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
            
            setInterval(() => this.updateUI(), 2000);
            
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
                if (message.sync_data && message.sync_data.type === 'trade_execution') {
                    console.log('Processing trade execution from enterprise BC');
                    this.handleTradeExecution(message.sync_data.trade);
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
        console.log('Asset:', trade.asset);
        console.log('Quantity:', trade.quantity);
        console.log('Price:', trade.price);
        console.log('Buyer:', trade.buyer, 'Network:', trade.buyer_network);
        console.log('Seller:', trade.seller, 'Network:', trade.seller_network);
        
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
        
        // Show notification
        this.showTradeNotification(trade);
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
            
            // Step 2: Place order and check for immediate execution
            this.orderBook.place_buy_order(this.userId, asset, quantityInt, priceInt);
            
            // Step 3: Check if trade occurred by looking at recent trades
            const recentTrades = JSON.parse(this.orderBook.get_recent_trades_json());
            const currentTime = Math.floor(Date.now() / 1000);
            
            // Find trades from the last 2 seconds that match our order
            const newTrade = recentTrades.find(trade => 
                Math.abs(trade.timestamp - currentTime) < 2 &&
                trade.buyer === this.userId &&
                trade.asset === asset
            );
            
            if (newTrade) {
                // Create execution block
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
            this.updateOrderBook();
            this.updateUI();
            
        } catch (error) {
            console.error('Error placing buy order:', error);
        }
    }

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
            
            // Step 2: Place order and check for immediate execution
            this.orderBook.place_sell_order(this.userId, asset, quantityInt, priceInt);
            
            // Step 3: Check if trade occurred by looking at recent trades
            const recentTrades = JSON.parse(this.orderBook.get_recent_trades_json());
            const currentTime = Math.floor(Date.now() / 1000);
            
            // Find trades from the last 2 seconds that match our order
            const newTrade = recentTrades.find(trade => 
                Math.abs(trade.timestamp - currentTime) < 2 &&
                trade.seller === this.userId &&
                trade.asset === asset
            );
            
            if (newTrade) {
                // Create execution block
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
            this.updateOrderBook();
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
            const orderBookData = JSON.parse(this.orderBook.get_order_book_json());
            const tradesData = JSON.parse(this.orderBook.get_recent_trades_json());

            this.updateTable('bids-tbody', orderBookData.bids || [], 'bid');
            this.updateTable('asks-tbody', orderBookData.asks || [], 'ask');
            this.updateTradesTable(tradesData || []);

        } catch (error) {
            console.error('Error updating order book:', error);
        }
    }

    updateTable(tableId, orders, type) {
        const tbody = document.getElementById(tableId);
        if (!tbody) return;

        if (orders.length === 0) {
            tbody.innerHTML = `<tr><td colspan="4">No ${type} orders</td></tr>`;
            return;
        }

        tbody.innerHTML = orders.map(order => `
            <tr>
                <td>$${(order.price / 100).toFixed(2)}</td>
                <td>${(order.quantity / 100).toFixed(2)}</td>
                <td>${order.asset || 'N/A'}</td>
                <td>${order.trader.substring(0, 8)}...</td>
            </tr>
        `).join('');
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

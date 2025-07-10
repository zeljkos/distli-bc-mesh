// Simplified app.js - Replaces main.js, blockchain-ui.js, mesh-manager.js, trading.js, wasm-loader.js
// Reduced from 1500+ lines to ~400 lines

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
        
        this.rtcConfig = {
            iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
        };
    }

    async init() {
        try {
            // Initialize WASM
            await init();
            this.blockchain = new Blockchain();
            this.orderBook = new OrderBook();
            
            // Add initial validator
            this.blockchain.add_validator(this.userId, 1000);
            
            // Setup UI
            this.setupEventListeners();
            this.setupDefaultServer();
            this.updateUI();
            
            // Start update loop
            setInterval(() => this.updateUI(), 2000);
            
            console.log('App initialized');
        } catch (error) {
            console.error('Init failed:', error);
            alert('Failed to initialize. Please refresh.');
        }
    }

    setupEventListeners() {
        // Connect button
        document.getElementById('connect-btn')?.addEventListener('click', () => {
            if (this.connected) {
                this.disconnect();
            } else {
                this.connect();
            }
        });

        // Network buttons
        document.getElementById('join-network-btn')?.addEventListener('click', () => this.joinNetwork());
        document.getElementById('refresh-networks-btn')?.addEventListener('click', () => this.refreshNetworks());
        document.getElementById('discover-btn')?.addEventListener('click', () => this.discoverPeers());
        document.getElementById('connect-all-btn')?.addEventListener('click', () => this.connectAllPeers());

        // Message
        document.getElementById('message-btn')?.addEventListener('click', () => this.sendMessage());
        document.getElementById('message-input')?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.sendMessage();
        });

        // Trading
        document.querySelector('.btn-buy')?.addEventListener('click', () => this.placeBuyOrder());
        document.querySelector('.btn-sell')?.addEventListener('click', () => this.placeSellOrder());
        document.querySelector('.refresh-btn')?.addEventListener('click', () => this.updateOrderBook());

        // Tabs
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

    // Connection Management
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
                this.updateUI();
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
                this.handleP2PBlock(message.block);
                break;
        }
    }

    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    // Network Management
    joinNetwork() {
        const networkId = this.getSelectedNetwork();
        if (networkId && this.connected) {
            this.send({ type: 'join_network', network_id: networkId });
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
        
        // Clear existing options except first
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

    // WebRTC P2P
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
            this.dataChannels.set(peerId, channel);
            this.updateUI();
        };
        
        channel.onclose = () => {
            this.dataChannels.delete(peerId);
            this.updateUI();
        };
        
        channel.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleP2PMessage(message, peerId);
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

    handleP2PMessage(message, fromPeer) {
        if (message.type === 'blockchain_block' && message.sender !== this.userId) {
            this.handleP2PBlock(message.block);
        }
    }

    handleP2PBlock(block) {
        if (!block || block.height <= 0) return;
        
        try {
            const currentHeight = this.blockchain.get_chain_length() - 1;
            if (block.height === currentHeight + 1) {
                const success = this.blockchain.add_p2p_block(JSON.stringify(block));
                if (success) {
                    console.log('Synced block #' + block.height);
                    this.updateUI();
                }
            }
        } catch (error) {
            console.error('Error processing P2P block:', error);
        }
    }

    broadcastToP2P(message) {
        let sent = 0;
        for (const [peerId, channel] of this.dataChannels) {
            if (channel.readyState === 'open') {
                try {
                    channel.send(JSON.stringify(message));
                    sent++;
                } catch (error) {
                    console.error('Failed to send to peer:', error);
                }
            }
        }
        return sent;
    }

    // Messaging
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
                    
                    // Broadcast to P2P
                    this.broadcastToP2P({
                        type: 'blockchain_block',
                        block: minedBlock,
                        sender: this.userId
                    });
                    
                    // Send to tracker
                    if (this.connected) {
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

    // Trading
    placeBuyOrder() {
        const asset = document.getElementById('buy-asset')?.value;
        const quantity = parseFloat(document.getElementById('buy-quantity')?.value) || 0;
        const price = parseFloat(document.getElementById('buy-price')?.value) || 0;

        if (!asset || quantity <= 0 || price <= 0) return;

        try {
            const quantityInt = Math.floor(quantity * 100);
            const priceInt = Math.floor(price * 100);

            this.orderBook.place_buy_order(this.userId, asset, quantityInt, priceInt);
            this.blockchain.call_contract_buy(asset, quantity, price, this.userId);
            this.blockchain.mine_block();

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

            this.orderBook.place_sell_order(this.userId, asset, quantityInt, priceInt);
            this.blockchain.call_contract_sell(asset, quantity, price, this.userId);
            this.blockchain.mine_block();

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
            tbody.innerHTML = `<tr><td colspan="3">No ${type} orders</td></tr>`;
            return;
        }

        tbody.innerHTML = orders.map(order => `
            <tr>
                <td>$${(order.price / 100).toFixed(2)}</td>
                <td>${(order.quantity / 100).toFixed(2)}</td>
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

    // UI Management
    updateUI() {
        this.updateConnectionStatus();
        this.updateStats();
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

    updateStats() {
        // Update any additional stats displays
    }

    updateButtonStates() {
        const connectBtn = document.getElementById('connect-btn');
        const joinBtn = document.getElementById('join-network-btn');
        const discoverBtn = document.getElementById('discover-btn');
        const connectAllBtn = document.getElementById('connect-all-btn');
        const messageBtn = document.getElementById('message-btn');

        if (connectBtn) {
            connectBtn.textContent = this.connected ? 'Disconnect' : 'Connect';
        }
        
        if (joinBtn) joinBtn.disabled = !this.connected;
        if (discoverBtn) discoverBtn.disabled = !this.currentNetwork;
        if (connectAllBtn) connectAllBtn.disabled = !this.currentNetwork || !this.availablePeers.length;
        if (messageBtn) messageBtn.disabled = !this.connected;
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
            
            if (chainLength > 1) {
                const latestBlockJson = this.blockchain.get_latest_block_json();
                if (latestBlockJson && latestBlockJson !== '{}') {
                    const block = JSON.parse(latestBlockJson);
                    if (block.height > 0) {
                        html += this.generateBlockDisplay(block);
                    }
                }
            } else {
                html += '<p>No blocks yet - send a message to create the first block</p>';
            }
            
            div.innerHTML = html;
            
        } catch (error) {
            div.innerHTML = `<p>Error loading blockchain: ${error.message}</p>`;
        }
    }

    generateBlockDisplay(block) {
        if (!block.transactions || block.transactions.length === 0) return '';

        const tx = block.transactions[0];
        let content = '';
        let typeLabel = 'Transaction';
        
        if (tx.tx_type?.Message) {
            content = tx.tx_type.Message.content;
            typeLabel = 'Message';
        } else if (tx.tx_type?.Trading) {
            const trading = tx.tx_type.Trading;
            content = `${trading.quantity / 100} ${trading.asset} @ $${trading.price / 100}`;
            typeLabel = 'Trade';
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
                    From: ${tx.from?.substring(0, 12)}... | Amount: ${tx.amount || 0}
                </div>
            </div>
        `;
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
        
        // Find and activate tab button
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

// Initialize app when DOM loads
document.addEventListener('DOMContentLoaded', async () => {
    try {
        const app = new DistliApp();
        await app.init();
        window.app = app; // For debugging
        console.log('App ready');
    } catch (error) {
        console.error('Failed to start app:', error);
        alert('Failed to start application. Please refresh.');
    }
});

// Make functions globally available for HTML onclick handlers
window.sendMessage = () => window.app?.sendMessage();
window.placeBuyOrder = () => window.app?.placeBuyOrder();
window.placeSellOrder = () => window.app?.placeSellOrder();
window.updateOrderBook = () => window.app?.updateOrderBook();

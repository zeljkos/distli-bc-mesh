// js/mesh-manager.js - Enhanced WebRTC P2P Manager with Offline Capabilities

class EnhancedMeshManager {
    constructor() {
        this.ws = null;
        this.connected = false;
        this.currentNetwork = null;
        this.dataChannels = new Map();
        this.peers = new Map();
        this.availablePeers = [];
        this.serverUrl = '';
        this.offlineMode = false;
        this.pendingMessages = [];
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.rtcConfig = {
            iceServers: [
                { urls: 'stun:stun.l.google.com:19302' },
                { urls: 'stun:stun1.l.google.com:19302' }
            ]
        };
        
        // Bind methods to maintain context
        this.handleMessage = this.handleMessage.bind(this);
        this.handleTrackerDisconnect = this.handleTrackerDisconnect.bind(this);
        this.handleP2PMessage = this.handleP2PMessage.bind(this);
    }

    async connect() {
        const server = document.getElementById('server-input').value.trim() || `${window.location.hostname}:3030`;
        this.serverUrl = server;
        const wsUrl = `ws://${server}/ws`;
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                this.connected = true;
                this.offlineMode = false;
                this.reconnectAttempts = 0;
                console.log('Connected to tracker');
                this.refreshNetworkList();
                this.processPendingMessages();
                this.notifyUI();
            };
            
            this.ws.onclose = () => {
                this.connected = false;
                console.log('Disconnected from tracker');
                this.handleTrackerDisconnect();
                this.notifyUI();
            };
            
            this.ws.onerror = () => {
                console.log('Tracker connection error');
                this.handleTrackerDisconnect();
            };
            
            this.ws.onmessage = (event) => {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            };
        } catch (error) {
            console.log('Failed to connect to tracker:', error);
            this.handleTrackerDisconnect();
        }
    }

    handleTrackerDisconnect() {
        this.connected = false;
        
        const activeChannels = Array.from(this.dataChannels.values())
            .filter(channel => channel.readyState === 'open');
        
        if (activeChannels.length > 0) {
            this.offlineMode = true;
            console.log(`Entering offline mode with ${activeChannels.length} P2P connections`);
            
            this.broadcastToP2P({
                type: 'tracker_disconnect',
                message: 'Tracker disconnected, continuing P2P only',
                timestamp: Math.floor(Date.now() / 1000)
            });
        } else {
            this.offlineMode = false;
            console.log('No P2P connections available - fully offline');
        }
        
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            setTimeout(() => {
                this.reconnectAttempts++;
                console.log(`Reconnection attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts}`);
                this.connect();
            }, 5000 * this.reconnectAttempts);
        }
    }

    sendMessage(messageText) {
        if (!messageText.trim()) return false;

        const messageData = {
            type: 'message',
            content: messageText,
            sender: this.getUserId(),
            timestamp: Math.floor(Date.now() / 1000),
            network: this.currentNetwork
        };

        // Always add to local blockchain first (auto-mines)
        try {
            window.blockchain.add_message(messageText, this.getUserId());
            console.log('Message added to local blockchain (auto-mined):', messageText);
        } catch (error) {
            console.error('Failed to add message to local blockchain:', error);
            return false;
        }

        let sent = false;

        // Try to send via tracker if connected
        if (this.connected && this.ws && this.ws.readyState === WebSocket.OPEN) {
            try {
                this.ws.send(JSON.stringify({
                    type: 'message',
                    content: messageText,
                    sender: this.getUserId(),
                    timestamp: Math.floor(Date.now() / 1000)
                }));
                sent = true;
                console.log('Message sent via tracker');
            } catch (error) {
                console.log('Failed to send via tracker:', error);
            }
        }

        // Try to send via P2P
        const p2pSent = this.broadcastToP2P(messageData);
        if (p2pSent > 0) {
            sent = true;
            console.log(`Message sent to ${p2pSent} P2P peers`);
        }

        // Queue for later if completely offline
        if (!sent) {
            this.pendingMessages.push(messageData);
            console.log('Message queued for when connection is restored');
            this.showPendingIndicator();
        }

        return true;
    }

    broadcastToP2P(message) {
        let sent = 0;
        const deadChannels = [];
        
        for (const [peerId, channel] of this.dataChannels) {
            if (channel.readyState === 'open') {
                try {
                    channel.send(JSON.stringify(message));
                    sent++;
                } catch (error) {
                    console.log(`Failed to send to ${peerId.substring(0, 8)}: ${error.message}`);
                    deadChannels.push(peerId);
                }
            } else {
                deadChannels.push(peerId);
            }
        }
        
        // Clean up dead channels
        deadChannels.forEach(peerId => {
            this.dataChannels.delete(peerId);
            this.peers.delete(peerId);
        });
        
        return sent;
    }

    broadcastTradingOrder(orderData) {
        const message = {
            type: 'order_transaction',
            ...orderData,
            timestamp: Math.floor(Date.now() / 1000)
        };
        
        const sent = this.broadcastToP2P(message);
        console.log(`Trading order broadcast to ${sent} P2P peers`);
        return sent;
    }

    processPendingMessages() {
        if (this.pendingMessages.length === 0) return;
        
        console.log(`Processing ${this.pendingMessages.length} pending messages`);
        
        const messages = [...this.pendingMessages];
        this.pendingMessages = [];
        
        messages.forEach(message => {
            if (this.connected && this.ws && this.ws.readyState === WebSocket.OPEN) {
                try {
                    this.ws.send(JSON.stringify(message));
                    console.log('Sent pending message via tracker');
                } catch (error) {
                    this.pendingMessages.push(message);
                }
            }
        });
        
        this.updatePendingIndicator();
    }

    handleP2PMessage(message, fromPeer) {
        console.log(`P2P message from ${fromPeer.substring(0, 8)}: ${message.type}`);
        
        switch (message.type) {
            case 'tracker_disconnect':
                console.log(`Peer ${fromPeer.substring(0, 8)} reports tracker disconnect`);
                break;
                
            case 'message':
                console.log(`P2P chat from ${fromPeer.substring(0, 8)}: "${message.content}"`);
                try {
                    // Auto-mines when added
                    window.blockchain.add_message(message.content, message.sender);
                    this.notifyUI();
                } catch (error) {
                    console.log(`Error adding P2P message: ${error.message}`);
                }
                break;
                
            case 'order_transaction':
                console.log(`P2P Trading Order from ${fromPeer.substring(0, 8)}: ${message.action}`);
                try {
                    if (message.action === 'buy') {
                        window.orderBook.place_buy_order(message.trader, message.asset, message.quantity, message.price);
                    } else if (message.action === 'sell') {
                        window.orderBook.place_sell_order(message.trader, message.asset, message.quantity, message.price);
                    }
                    
                    // Create blockchain transaction (auto-mines)
                    const tradingMessage = `${message.action.toUpperCase()}_ORDER: ${message.quantity / 100} ${message.asset} @ $${message.price / 100}`;
                    window.blockchain.add_message(tradingMessage, message.trader);
                    
                    console.log(`Synchronized P2P order: ${message.action} ${message.quantity / 100} ${message.asset} @ $${message.price / 100}`);
                    
                    // Notify other modules
                    window.dispatchEvent(new CustomEvent('orderBookUpdate'));
                    this.notifyUI();
                } catch (error) {
                    console.log(`Error processing P2P order: ${error.message}`);
                }
                break;
                
            case 'blockchain_block':
                console.log(`P2P block from ${fromPeer.substring(0, 8)}`);
                if (message.block && window.blockchain) {
                    const receivedBlock = message.block;
                    const currentHeight = window.blockchain.get_chain_length() - 1;
                    
                    if (receivedBlock.height === currentHeight + 1) {
                        if (window.blockchain.add_p2p_block(JSON.stringify(receivedBlock))) {
                            console.log(`Synced block #${receivedBlock.height}`);
                            this.notifyUI();
                        }
                    }
                }
                break;
        }
    }

    getConnectionStatus() {
        const p2pConnections = this.dataChannels.size;
        const trackerConnected = this.connected;
        
        if (trackerConnected && p2pConnections > 0) {
            return { status: 'fully_connected', connections: p2pConnections };
        } else if (p2pConnections > 0) {
            return { status: 'p2p_only', connections: p2pConnections };
        } else if (trackerConnected) {
            return { status: 'tracker_only', connections: 0 };
        } else {
            return { status: 'offline', connections: 0 };
        }
    }

    showPendingIndicator() {
        this.updatePendingIndicator();
    }

    updatePendingIndicator() {
        const pendingCount = document.getElementById('pending-count');
        const pendingMessages = document.getElementById('pending-messages');
        
        if (this.pendingMessages.length > 0) {
            pendingCount.style.display = 'block';
            pendingMessages.textContent = this.pendingMessages.length;
        } else {
            pendingCount.style.display = 'none';
        }
    }

    notifyUI() {
        window.dispatchEvent(new CustomEvent('meshUpdate', {
            detail: this.getConnectionStatus()
        }));
    }

    getUserId() {
        if (!window.userId) {
            window.userId = 'user_' + Math.random().toString(36).substr(2, 9);
        }
        return window.userId;
    }

    // WebRTC Connection Methods
    handleMessage(message) {
        switch (message.type) {
            case 'network_list_update':
                this.updateNetworkDropdown(message.networks);
                break;
            case 'network_info':
                this.currentNetwork = message.network_id;
                console.log('Joined network:', message.network_id);
                this.notifyUI();
                break;
            case 'peers':
                this.availablePeers = message.peers;
                console.log('Available peers:', message.peers.length);
                this.notifyUI();
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
        }
    }

    async connectToAllPeers() {
        if (!this.availablePeers || this.availablePeers.length === 0) {
            console.log('No peers available to connect to');
            return;
        }

        console.log(`Connecting to ${this.availablePeers.length} peers...`);
        
        for (const peerId of this.availablePeers) {
            if (!this.dataChannels.has(peerId)) {
                await this.connectToPeer(peerId);
            }
        }
    }

    async connectToPeer(peerId) {
        try {
            console.log(`Initiating connection to peer: ${peerId.substring(0, 8)}`);
            
            const pc = new RTCPeerConnection(this.rtcConfig);
            this.peers.set(peerId, pc);
            
            const channel = pc.createDataChannel('blockchain', { ordered: true });
            this.setupDataChannel(channel, peerId);
            
            pc.onicecandidate = (event) => {
                if (event.candidate) {
                    this.send({
                        type: 'candidate',
                        target: peerId,
                        candidate: event.candidate
                    });
                }
            };
            
            const offer = await pc.createOffer();
            await pc.setLocalDescription(offer);
            
            this.send({
                type: 'offer',
                target: peerId,
                offer: offer
            });
            
        } catch (error) {
            console.log(`Failed to connect to peer ${peerId}: ${error.message}`);
        }
    }

    setupDataChannel(channel, peerId) {
        channel.onopen = () => {
            console.log(`Data channel opened with: ${peerId.substring(0, 8)}`);
            this.dataChannels.set(peerId, channel);
            this.notifyUI();
        };
        
        channel.onclose = () => {
            console.log(`Data channel closed with: ${peerId.substring(0, 8)}`);
            this.dataChannels.delete(peerId);
            this.notifyUI();
        };
        
        channel.onmessage = (event) => {
            this.handleP2PMessage(JSON.parse(event.data), peerId);
        };
        
        channel.onerror = (error) => {
            console.log(`Data channel error with ${peerId}: ${error}`);
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
                    this.send({
                        type: 'candidate',
                        target: fromPeer,
                        candidate: event.candidate
                    });
                }
            };
            
            await pc.setRemoteDescription(offer);
            const answer = await pc.createAnswer();
            await pc.setLocalDescription(answer);
            
            this.send({
                type: 'answer',
                target: fromPeer,
                answer: answer
            });
            
        } catch (error) {
            console.log(`Failed to handle offer from ${fromPeer}: ${error.message}`);
        }
    }

    async handleAnswer(fromPeer, answer) {
        try {
            const pc = this.peers.get(fromPeer);
            if (pc) {
                await pc.setRemoteDescription(answer);
                console.log(`Connection established with: ${fromPeer.substring(0, 8)}`);
            }
        } catch (error) {
            console.log(`Failed to handle answer from ${fromPeer}: ${error.message}`);
        }
    }

    async handleCandidate(fromPeer, candidate) {
        try {
            const pc = this.peers.get(fromPeer);
            if (pc) {
                await pc.addIceCandidate(candidate);
            }
        } catch (error) {
            console.log(`Failed to handle ICE candidate from ${fromPeer}: ${error.message}`);
        }
    }

    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    joinNetwork(networkId) {
        if (this.connected && networkId) {
            this.send({
                type: 'join_network',
                network_id: networkId
            });
            console.log('Joining network:', networkId);
        }
    }

    getSelectedNetwork() {
        const selectValue = document.getElementById('network-select').value;
        const inputValue = document.getElementById('network-input').value.trim();
        return selectValue || inputValue || null;
    }

    async refreshNetworkList() {
        if (!this.serverUrl) return;
        try {
            const response = await fetch(`http://${this.serverUrl}/api/network-list`);
            const networks = await response.json();
            this.updateNetworkDropdown(networks);
            console.log('Network list refreshed');
        } catch (error) {
            console.error('Failed to fetch network list:', error);
        }
    }

    updateNetworkDropdown(networks) {
        const select = document.getElementById('network-select');
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
}

export { EnhancedMeshManager };

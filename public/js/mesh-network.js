// MeshManager class with WebRTC implementation
class MeshManager {
    constructor() {
        this.ws = null;
        this.peers = new Map();
        this.dataChannels = new Map();
        this.availablePeers = [];
        this.connected = false;
        this.currentNetwork = null;
        this.networkPeerCount = 0;
        this.serverUrl = '';
        this.peerId = null;
    }

    // NEW: Remove pending transactions for consumed orders
    removePendingTransactionsForAsset(asset, quantity, orderType) {
        if (!window.blockchain) return;
        
        log(`🧹 Looking for pending transactions to remove: ${orderType} ${asset} (consumed qty: ${quantity})`);
        
        const originalCount = window.blockchain.pending.length;
        log(`📊 Original pending count: ${originalCount}`);
        
        // Show what we're looking through
        window.blockchain.pending.forEach((tx, i) => {
            if (tx.type === 'contract_call' && tx.call && tx.call.contract_id === 'trading_contract') {
                log(`🔍 Checking pending ${i}: ${tx.call.function}(${JSON.stringify(tx.call.params)})`);
            }
        });
        
        // Remove pending contract calls that match the consumed order
        window.blockchain.pending = window.blockchain.pending.filter(tx => {
            if (tx.type !== 'contract_call') {
                log(`✅ Keeping non-contract transaction: ${tx.type}`);
                return true; // Keep non-contract transactions
            }
            
            const call = tx.call;
            if (!call || call.contract_id !== 'trading_contract') {
                log(`✅ Keeping non-trading contract call`);
                return true; // Keep non-trading transactions
            }
            
            // Check if this transaction created an order for the same asset and type
            const isMatchingOrder = (
                call.function === orderType && 
                call.params.asset === asset
                // Remove ANY pending order for this asset/type since cross-network trade happened
            );
            
            if (isMatchingOrder) {
                log(`🗑️ REMOVING pending transaction: ${call.function}(${JSON.stringify(call.params)}) - matches consumed ${orderType} ${asset}`);
                return false; // Remove this transaction
            }
            
            log(`✅ Keeping different transaction: ${call.function}(${JSON.stringify(call.params)})`);
            return true; // Keep this transaction
        });
        
        const removedCount = originalCount - window.blockchain.pending.length;
        log(`📊 New pending count: ${window.blockchain.pending.length}`);
        
        if (removedCount > 0) {
            log(`✅ Removed ${removedCount} pending transaction(s) for consumed ${orderType} order`);
            window.blockchain.saveToStorage();
            
            // Update UI to reflect the change
            if (typeof updateUI === 'function') {
                setTimeout(() => {
                    updateUI();
                }, 100);
            }
        } else {
            log(`⚠️ No matching pending transactions found to remove for ${orderType} ${asset}`);
        }
    }
    
    async connect() {
        try {
            const serverInput = document.getElementById('server-input').value.trim();
            const server = serverInput || `${window.location.hostname}:3030`;
            this.serverUrl = server;
            const wsUrl = `ws://${server}/ws`;
            
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                this.connected = true;
                log('Connected to tracker');
                this.refreshNetworkList();
                updateUI();
            };
            
            this.ws.onclose = () => {
                this.connected = false;
                log('Disconnected from tracker');
                updateUI();
            };
            
            this.ws.onmessage = (event) => {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            };
            
        } catch (error) {
            log('Connection failed: ' + error.message);
        }
    }

    handleMessage(message) {
        switch (message.type) {
            case 'network_list_update':
                this.updateNetworkDropdown(message.networks);
                break;
                
            case 'network_info':
                this.currentNetwork = message.network_id;
                this.networkPeerCount = message.peer_count;
                log('Joined network: ' + message.network_id);
                if (window.blockchain) {
                    window.blockchain.loadFromStorage();
                }
                updateUI();
                setTimeout(() => {
                    if (typeof updateOrderBookFromContract === 'function') {
                        updateOrderBookFromContract();
                    }
                }, 100);
                break;
                
            case 'peers':
                this.availablePeers = message.peers;
                log('Available peers: ' + message.peers.length);
                updateUI();
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
                if (window.blockchain) {
                    window.blockchain.addBlock(message.block);
                    log('Received block #' + message.block.id);
                    updateUI();
                    if (typeof updateOrderBookFromContract === 'function') {
                        updateOrderBookFromContract();
                    }
                }
                break;
                
            case 'transaction':
                if (window.blockchain) {
                    if (message.transaction.type === 'message') {
                        window.blockchain.addMessage(message.transaction.data, message.transaction.sender, true);
                    } else if (message.transaction.type === 'contract_call') {
                        if (message.transaction.result && message.transaction.result.success) {
                            window.blockchain.contract_vm.apply_state_changes(
                                message.transaction.call.contract_id, 
                                message.transaction.result.state_changes
                            );
                        }
                        window.blockchain.pending.push(message.transaction);
                    }
                    log('Received transaction');
                    updateUI();
                    if (typeof updateOrderBookFromContract === 'function') {
                        updateOrderBookFromContract();
                    }
                }
                break;
                
            case 'cross_network_trade':
                this.handleCrossNetworkTrade(message);
                break;
        }
    }

    handleCrossNetworkTrade(message) {
        log(`Cross-network trade executed: ${message.quantity} ${message.asset} @ ${message.price}`);
        log(`Between networks: ${message.buyer_network} ↔ ${message.seller_network}`);

        const currentNetwork = this.currentNetwork;
        const isBuyerNetwork = currentNetwork === message.buyer_network;
        const isSellerNetwork = currentNetwork === message.seller_network;
        
        if (!isBuyerNetwork && !isSellerNetwork) {
            log('Cross-network trade does not involve this network');
            return;
        }

        if (!window.blockchain) return;

        try {
            const tradingContract = window.blockchain.contract_vm.contracts.get("trading_contract");
            if (!tradingContract || !tradingContract.state) {
                log('No trading contract found for cross-network trade update');
                return;
            }

            let orderRemoved = false;
            const state = tradingContract.state;
            
            if (isBuyerNetwork) {
                log(`This network (${currentNetwork}) was the BUYER`);
                
                const bids = state.orderBook?.bids || [];
                log(`Looking for buy order to remove - Asset: ${message.asset}, Price: ${message.price}, Quantity: ${message.quantity}`);
                log(`Current bids: ${JSON.stringify(bids.map(o => ({asset: o.asset, price: o.price, quantity: o.quantity})))}`);
                
                for (let i = bids.length - 1; i >= 0; i--) {
                    const order = bids[i];
                    log(`Checking bid ${i}: ${order.asset} ${order.quantity}@${order.price} vs trade ${message.asset} ${message.quantity}@${message.price}`);
                    
                    // More flexible matching - find the best matching order
                    if (order.asset === message.asset && 
                        order.type === 'buy' && 
                        order.price >= message.price) {
                        
                        log(`Found matching buy order: ${order.quantity} ${order.asset} @ ${order.price}`);
                        
                        if (order.quantity <= message.quantity) {
                            // Remove entire order if it's smaller or equal
                            log(`Removing entire buy order: ${order.quantity} ${order.asset} @ ${order.price}`);
                            bids.splice(i, 1);
                        } else {
                            // Reduce order quantity if it's larger
                            order.quantity -= message.quantity;
                            log(`Reduced buy order to: ${order.quantity} ${order.asset} @ ${order.price}`);
                        }
                        orderRemoved = true;
                        break;
                    }
                }
                
                if (!orderRemoved) {
                    log(`⚠️ No matching buy order found for cross-network trade`);
                }
            }
            
            if (isSellerNetwork) {
                log(`This network (${currentNetwork}) was the SELLER`);
                
                const asks = state.orderBook?.asks || [];
                log(`Looking for sell order to remove - Asset: ${message.asset}, Price: ${message.price}, Quantity: ${message.quantity}`);
                log(`Current asks: ${JSON.stringify(asks.map(o => ({asset: o.asset, price: o.price, quantity: o.quantity})))}`);
                
                for (let i = asks.length - 1; i >= 0; i--) {
                    const order = asks[i];
                    log(`Checking ask ${i}: ${order.asset} ${order.quantity}@${order.price} vs trade ${message.asset} ${message.quantity}@${message.price}`);
                    
                    // More flexible matching - find the best matching order
                    if (order.asset === message.asset && 
                        order.type === 'sell' && 
                        order.price <= message.price) {
                        
                        log(`Found matching sell order: ${order.quantity} ${order.asset} @ ${order.price}`);
                        
                        if (order.quantity <= message.quantity) {
                            // Remove entire order if it's smaller or equal
                            log(`Removing entire sell order: ${order.quantity} ${order.asset} @ ${order.price}`);
                            asks.splice(i, 1);
                        } else {
                            // Reduce order quantity if it's larger
                            order.quantity -= message.quantity;
                            log(`Reduced sell order to: ${order.quantity} ${order.asset} @ ${order.price}`);
                        }
                        orderRemoved = true;
                        break;
                    }
                }
                
                if (!orderRemoved) {
                    log(`⚠️ No matching sell order found for cross-network trade`);
                }
            }

            if (orderRemoved) {
                if (!state.trades) {
                    state.trades = [];
                }
                
                const crossNetworkTrade = {
                    id: message.trade_id,
                    asset: message.asset,
                    quantity: message.quantity,
                    price: message.price,
                    buyer: `${message.buyer_network}_network`,
                    seller: `${message.seller_network}_network`,
                    timestamp: message.timestamp * 1000,
                    cross_network: true
                };
                
                state.trades.push(crossNetworkTrade);
                tradingContract.state = state;
                window.blockchain.saveToStorage();
                
                // IMPORTANT: Remove pending transactions that created the consumed orders
                this.removePendingTransactionsForAsset(message.asset, message.quantity, isBuyerNetwork ? 'buy' : 'sell');
                
                // Save the updated state but DON'T trigger blockchain update
                // (cross-network trades are notifications, not new blocks to send)
                window.blockchain.saveToStorage();
                
                if (typeof updateOrderBookFromContract === 'function') {
                    updateOrderBookFromContract();
                }
                
                // Force UI update after a brief delay to ensure state is saved
                setTimeout(() => {
                    updateUI();
                    if (typeof updateOrderBookFromContract === 'function') {
                        updateOrderBookFromContract();
                    }
                }, 500);
                
                log(`✅ Local order book updated for cross-network trade`);
                
                // For cross-network trade notifications, we ONLY update the order book and trades
                // We do NOT add pending transactions since the trade is already executed by enterprise BC
                log(`📝 Cross-network trade notification processed - no mining required`);
                
            } else {
                log(`⚠️ No matching local order found for cross-network trade`);
            }
            
        } catch (error) {
            log(`❌ Error updating order book for cross-network trade: ${error.message}`);
            console.error('Cross-network trade update error:', error);
        }
    }

    async connectToPeer(peerId) {
        if (this.peers.has(peerId)) {
            log(`Already connected to peer ${peerId.substring(0, 8)}`);
            return;
        }

        try {
            const peerConnection = new RTCPeerConnection({
                iceServers: [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' }
                ]
            });

            this.peers.set(peerId, peerConnection);

            peerConnection.onicecandidate = (event) => {
                if (event.candidate) {
                    this.send({
                        type: 'candidate',
                        target: peerId,
                        candidate: event.candidate
                    });
                }
            };

            const dataChannel = peerConnection.createDataChannel('blockchain', {
                ordered: true
            });
            
            this.setupDataChannel(dataChannel, peerId);

            const offer = await peerConnection.createOffer();
            await peerConnection.setLocalDescription(offer);

            this.send({
                type: 'offer',
                target: peerId,
                offer: offer
            });

            log(`Sent connection offer to peer ${peerId.substring(0, 8)}`);
        } catch (error) {
            log(`Failed to connect to peer ${peerId.substring(0, 8)}: ${error.message}`);
            this.peers.delete(peerId);
        }
    }

    async handleOffer(peerId, offer) {
        try {
            const peerConnection = new RTCPeerConnection({
                iceServers: [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' }
                ]
            });

            this.peers.set(peerId, peerConnection);

            peerConnection.onicecandidate = (event) => {
                if (event.candidate) {
                    this.send({
                        type: 'candidate',
                        target: peerId,
                        candidate: event.candidate
                    });
                }
            };

            peerConnection.ondatachannel = (event) => {
                const dataChannel = event.channel;
                this.setupDataChannel(dataChannel, peerId);
            };

            await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
            const answer = await peerConnection.createAnswer();
            await peerConnection.setLocalDescription(answer);

            this.send({
                type: 'answer',
                target: peerId,
                answer: answer
            });

            log(`Sent connection answer to peer ${peerId.substring(0, 8)}`);
        } catch (error) {
            log(`Failed to handle offer from peer ${peerId.substring(0, 8)}: ${error.message}`);
            this.peers.delete(peerId);
        }
    }

    async handleAnswer(peerId, answer) {
        try {
            const peerConnection = this.peers.get(peerId);
            if (!peerConnection) {
                log(`No peer connection found for ${peerId.substring(0, 8)}`);
                return;
            }

            await peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
            log(`Received connection answer from peer ${peerId.substring(0, 8)}`);
        } catch (error) {
            log(`Failed to handle answer from peer ${peerId.substring(0, 8)}: ${error.message}`);
        }
    }

    async handleCandidate(peerId, candidate) {
        try {
            const peerConnection = this.peers.get(peerId);
            if (!peerConnection) {
                log(`No peer connection found for candidate from ${peerId.substring(0, 8)}`);
                return;
            }

            await peerConnection.addIceCandidate(new RTCIceCandidate(candidate));
        } catch (error) {
            log(`Failed to add ICE candidate from peer ${peerId.substring(0, 8)}: ${error.message}`);
        }
    }

    setupDataChannel(dataChannel, peerId) {
        dataChannel.onopen = () => {
            log(`Data channel opened with peer ${peerId.substring(0, 8)}`);
            this.dataChannels.set(peerId, dataChannel);
            updateUI();
        };

        dataChannel.onclose = () => {
            log(`Data channel closed with peer ${peerId.substring(0, 8)}`);
            this.dataChannels.delete(peerId);
            this.peers.delete(peerId);
            updateUI();
        };

        dataChannel.onerror = (error) => {
            log(`Data channel error with peer ${peerId.substring(0, 8)}: ${error}`);
        };

        dataChannel.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (error) {
                log(`Failed to parse message from peer ${peerId.substring(0, 8)}: ${error.message}`);
            }
        };
    }

    async connectToAll() {
        if (!this.availablePeers || this.availablePeers.length === 0) {
            log('No available peers to connect to');
            return;
        }

        log(`Connecting to ${this.availablePeers.length} peers...`);
        for (const peerId of this.availablePeers) {
            if (!this.dataChannels.has(peerId)) {
                await this.connectToPeer(peerId);
                await new Promise(resolve => setTimeout(resolve, 100));
            }
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

    async refreshNetworkList() {
        if (!this.serverUrl) return;
        try {
            const response = await fetch(`http://${this.serverUrl}/api/network-list`);
            const networks = await response.json();
            this.updateNetworkDropdown(networks);
        } catch (error) {
            log('Failed to fetch network list: ' + error.message);
        }
    }

    getSelectedNetwork() {
        const selectValue = document.getElementById('network-select')?.value;
        const inputValue = document.getElementById('network-input')?.value.trim();
        return selectValue || inputValue || null;
    }

    joinNetwork(networkId) {
        if (this.connected && networkId) {
            this.send({
                type: 'join_network',
                network_id: networkId
            });
            log('Joining network: ' + networkId);
        }
    }

    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
            
            if (message.type === 'blockchain_update') {
                log(`Sending blockchain update with ${message.new_blocks.length} blocks`);
            }
        } else {
            log('Cannot send message - WebSocket not open');
        }
    }

    broadcast(message) {
        this.dataChannels.forEach((dc) => {
            if (dc.readyState === 'open') {
                dc.send(JSON.stringify(message));
            }
        });
    }
}

// Test blockchain sync function
function testBlockchainSync() {
    if (!window.mesh || !window.mesh.connected || !window.mesh.currentNetwork) {
        log('Not connected to network - cannot sync');
        return;
    }
    
    log('Testing blockchain sync to enterprise...');
    if (window.blockchain) {
        window.blockchain.sendBlockchainUpdate();
    }
}

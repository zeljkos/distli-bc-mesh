// js/trading.js - Clean Trading with Automatic Blockchain Integration

class TradingManager {
    constructor() {
        this.mesh = null;
        this.blockchain = null;
        this.orderBook = null;

        // Bind methods to maintain context
        this.placeBuyOrder = this.placeBuyOrder.bind(this);
        this.placeSellOrder = this.placeSellOrder.bind(this);
        this.updateOrderBook = this.updateOrderBook.bind(this);

        // Wait for WASM and mesh to be ready
        window.addEventListener('wasmReady', (event) => {
            this.blockchain = event.detail.blockchain;
            this.orderBook = event.detail.orderBook;
            console.log('Trading manager connected to blockchain');
        });

        window.addEventListener('meshReady', (event) => {
            this.mesh = event.detail.mesh;
            console.log('Trading manager connected to mesh');
        });

        // Listen for order book updates from P2P
        window.addEventListener('orderBookUpdate', () => {
            this.updateOrderBook();
        });
    }

    placeBuyOrder() {
        const asset = document.getElementById('buy-asset').value;
        const quantity = parseFloat(document.getElementById('buy-quantity').value) || 0;
        const price = parseFloat(document.getElementById('buy-price').value) || 0;

        if (!asset || quantity <= 0 || price <= 0) {
            this.showAlert('Please enter valid asset, quantity, and price', 'error');
            return;
        }

        if (!this.blockchain || !this.orderBook) {
            this.showAlert('Blockchain not ready yet. Please wait.', 'warning');
            return;
        }

        try {
            // Convert to integers for WASM compatibility (2 decimal places)
            const quantityInt = Math.floor(quantity * 100);
            const priceInt = Math.floor(price * 100);

            // Step 1: Add to OrderBook for immediate UI update
            const orderId = this.orderBook.place_buy_order(this.getUserId(), asset, quantityInt, priceInt);
            console.log('OrderBook: Buy order placed -', quantity, asset, '@', price);

            // Step 2: Use new automatic blockchain method
            const txId = this.blockchain.call_contract_buy(asset, quantity, price, this.getUserId());
            console.log('Blockchain: Buy order processed automatically');

            // Step 3: Broadcast to P2P peers for synchronization
            if (this.mesh) {
                this.mesh.broadcastTradingOrder({
                    action: 'buy',
                    trader: this.getUserId(),
                    asset: asset,
                    quantity: quantityInt,
                    price: priceInt,
                    orderId: orderId,
                    txId: txId
                });
                console.log('P2P: Buy order broadcast to peers');
            }

            // Step 4: Clear form and update UI
            this.clearBuyForm();
            this.updateOrderBook();
            this.notifyUI();

            this.showAlert(`Buy order placed: ${quantity} ${asset} @ $${price}`, 'success');

        } catch (error) {
            console.error('Error placing buy order:', error);
            this.showAlert(`Failed to place buy order: ${error.message}`, 'error');
        }
    }

    placeSellOrder() {
        const asset = document.getElementById('sell-asset').value;
        const quantity = parseFloat(document.getElementById('sell-quantity').value) || 0;
        const price = parseFloat(document.getElementById('sell-price').value) || 0;

        if (!asset || quantity <= 0 || price <= 0) {
            this.showAlert('Please enter valid asset, quantity, and price', 'error');
            return;
        }

        if (!this.blockchain || !this.orderBook) {
            this.showAlert('Blockchain not ready yet. Please wait.', 'warning');
            return;
        }

        try {
            // Convert to integers for WASM compatibility
            const quantityInt = Math.floor(quantity * 100);
            const priceInt = Math.floor(price * 100);

            // Step 1: Add to OrderBook for immediate UI update
            const orderId = this.orderBook.place_sell_order(this.getUserId(), asset, quantityInt, priceInt);
            console.log('OrderBook: Sell order placed -', quantity, asset, '@', price);

            // Step 2: Use new automatic blockchain method
            const txId = this.blockchain.call_contract_sell(asset, quantity, price, this.getUserId());
            console.log('Blockchain: Sell order processed automatically');

            // Step 3: Broadcast to P2P peers for synchronization
            if (this.mesh) {
                this.mesh.broadcastTradingOrder({
                    action: 'sell',
                    trader: this.getUserId(),
                    asset: asset,
                    quantity: quantityInt,
                    price: priceInt,
                    orderId: orderId,
                    txId: txId
                });
                console.log('P2P: Sell order broadcast to peers');
            }

            // Step 4: Clear form and update UI
            this.clearSellForm();
            this.updateOrderBook();
            this.notifyUI();

            this.showAlert(`Sell order placed: ${quantity} ${asset} @ $${price}`, 'success');

        } catch (error) {
            console.error('Error placing sell order:', error);
            this.showAlert(`Failed to place sell order: ${error.message}`, 'error');
        }
    }

    updateOrderBook() {
        if (!this.orderBook) return;

        try {
            const orderBookData = JSON.parse(this.orderBook.get_order_book_json());
            const tradesData = JSON.parse(this.orderBook.get_recent_trades_json());

            // Update bids (buy orders)
            this.updateBidsTable(orderBookData.bids || []);

            // Update asks (sell orders)
            this.updateAsksTable(orderBookData.asks || []);

            // Update recent trades
            this.updateTradesTable(tradesData || []);

            console.log('Order book updated:', (orderBookData.bids?.length || 0), 'bids,', (orderBookData.asks?.length || 0), 'asks');

        } catch (error) {
            console.error('Error updating order book:', error);
        }
    }

    updateBidsTable(bids) {
        const bidsTableBody = document.getElementById('bids-tbody');
        if (!bidsTableBody) return;

        if (bids.length > 0) {
            bidsTableBody.innerHTML = bids.map(order => `
                <tr class="bid-row">
                    <td>$${(order.price / 100).toFixed(2)}</td>
                    <td>${(order.quantity / 100).toFixed(2)}</td>
                    <td title="${order.trader}">${order.trader.substring(0, 8)}...</td>
                </tr>
            `).join('');
        } else {
            bidsTableBody.innerHTML = '<tr><td colspan="3" style="text-align: center; color: #666;">No buy orders</td></tr>';
        }
    }

    updateAsksTable(asks) {
        const asksTableBody = document.getElementById('asks-tbody');
        if (!asksTableBody) return;

        if (asks.length > 0) {
            asksTableBody.innerHTML = asks.map(order => `
                <tr class="ask-row">
                    <td>$${(order.price / 100).toFixed(2)}</td>
                    <td>${(order.quantity / 100).toFixed(2)}</td>
                    <td title="${order.trader}">${order.trader.substring(0, 8)}...</td>
                </tr>
            `).join('');
        } else {
            asksTableBody.innerHTML = '<tr><td colspan="3" style="text-align: center; color: #666;">No sell orders</td></tr>';
        }
    }

    updateTradesTable(trades) {
        const tradesTableBody = document.getElementById('trades-tbody');
        if (!tradesTableBody) return;

        if (trades.length > 0) {
            tradesTableBody.innerHTML = trades.map(trade => `
                <tr>
                    <td>${new Date(trade.timestamp * 1000).toLocaleTimeString()}</td>
                    <td><strong>${trade.asset}</strong></td>
                    <td>${(trade.quantity / 100).toFixed(2)}</td>
                    <td>$${(trade.price / 100).toFixed(2)}</td>
                    <td title="${trade.buyer}">${trade.buyer.substring(0, 8)}...</td>
                    <td title="${trade.seller}">${trade.seller.substring(0, 8)}...</td>
                </tr>
            `).join('');
        } else {
            tradesTableBody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: #666;">No recent trades</td></tr>';
        }
    }

    clearBuyForm() {
        const quantityInput = document.getElementById('buy-quantity');
        const priceInput = document.getElementById('buy-price');

        if (quantityInput) quantityInput.value = '';
        if (priceInput) priceInput.value = '';
    }

    clearSellForm() {
        const quantityInput = document.getElementById('sell-quantity');
        const priceInput = document.getElementById('sell-price');

        if (quantityInput) quantityInput.value = '';
        if (priceInput) priceInput.value = '';
    }

    // Remove auto-mining since it's now automatic in blockchain
    // No longer needed - mining happens automatically when transactions are added

    showAlert(message, type = 'info') {
        // Create or update alert element
        let alert = document.getElementById('trading-alert');
        if (!alert) {
            alert = document.createElement('div');
            alert.id = 'trading-alert';
            alert.style.cssText = `
                position: fixed;
                top: 80px;
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

        // Auto-hide after 4 seconds
        setTimeout(() => {
            if (alert.parentNode) {
                alert.style.display = 'none';
            }
        }, 4000);
    }

    notifyUI() {
        window.dispatchEvent(new CustomEvent('tradingUpdate'));
    }

    getUserId() {
        if (!window.userId) {
            window.userId = 'user_' + Math.random().toString(36).substr(2, 9);
        }
        return window.userId;
    }

    // Contract-related functions (placeholders for future enhancement)
    deployTradingContract() {
        console.log('Deploying trading contract...');
        const resultsDiv = document.getElementById('contract-results');
        if (resultsDiv) {
            resultsDiv.textContent = 'Trading contract deployed successfully!\nContract Address: 0x' + Math.random().toString(16).substr(2, 8);
        }
    }

    placeContractBuyOrder() {
        console.log('Placing contract buy order...');
        // Implementation for smart contract buy order
    }

    placeContractSellOrder() {
        console.log('Placing contract sell order...');
        // Implementation for smart contract sell order
    }

    getContractOrderBook() {
        console.log('Getting contract order book...');
        // Implementation for contract order book retrieval
    }

    getContractTrades() {
        console.log('Getting contract trades...');
        // Implementation for contract trades retrieval
    }

    // Initialize trading manager
    init() {
        console.log('Trading manager initialized');

        // Set up initial order book
        setTimeout(() => {
            this.updateOrderBook();
        }, 1000);
    }
}

// Create global trading manager instance
const tradingManager = new TradingManager();

// Export functions for global access
window.placeBuyOrder = tradingManager.placeBuyOrder;
window.placeSellOrder = tradingManager.placeSellOrder;
window.updateOrderBook = tradingManager.updateOrderBook;
window.deployTradingContract = tradingManager.deployTradingContract;
window.placeContractBuyOrder = tradingManager.placeContractBuyOrder;
window.placeContractSellOrder = tradingManager.placeContractSellOrder;
window.getContractOrderBook = tradingManager.getContractOrderBook;
window.getContractTrades = tradingManager.getContractTrades;

export { tradingManager, TradingManager };

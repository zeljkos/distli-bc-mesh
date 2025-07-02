// Trading Functions and UI Management

// Initialize trading UI
function initializeTradingUI() {
    setupTradingTab();
    setupOrderBookTab();
}

function setupTradingTab() {
    const tradingTab = document.getElementById('trading-tab');
    if (tradingTab) {
        tradingTab.innerHTML = `
            <div class="trading-grid">
                <div>
                    <div class="order-form">
                        <h4>Buy Order</h4>
                        <select id="buy-asset">
                            <option value="BTC">BTC</option>
                            <option value="ETH">ETH</option>
                            <option value="ADA">ADA</option>
                        </select>
                        <input type="number" id="buy-quantity" placeholder="Quantity" step="0.001" min="0">
                        <input type="number" id="buy-price" placeholder="Price" step="0.01" min="0">
                        <button class="buy" onclick="placeBuyOrder()">Place Buy Order</button>
                    </div>
                </div>
                
                <div>
                    <div class="order-form">
                        <h4>Sell Order</h4>
                        <select id="sell-asset">
                            <option value="BTC">BTC</option>
                            <option value="ETH">ETH</option>
                            <option value="ADA">ADA</option>
                        </select>
                        <input type="number" id="sell-quantity" placeholder="Quantity" step="0.001" min="0">
                        <input type="number" id="sell-price" placeholder="Price" step="0.01" min="0">
                        <button class="sell" onclick="placeSellOrder()">Place Sell Order</button>
                    </div>
                </div>
            </div>

            <h4>Smart Contract Actions</h4>
            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 15px; margin-bottom: 20px;">
                <button onclick="callPythonPredictor()" style="background: #3776AB;">Python ML Predict</button>
                <button onclick="callWasmDeFi()" style="background: #654FF0;">WASM DeFi Pool</button>
                <button onclick="callJavaScriptAnalytics()" style="background: #F7DF1E; color: black;">JS Analytics</button>
            </div>

            <h4>Recent Trades</h4>
            <div id="recent-trades" class="order-book">
                <table>
                    <thead>
                        <tr><th>Time</th><th>Asset</th><th>Quantity</th><th>Price</th><th>Buyer</th><th>Seller</th></tr>
                    </thead>
                    <tbody id="trades-tbody">
                        <tr><td colspan="6">No trades yet - place some orders</td></tr>
                    </tbody>
                </table>
            </div>
        `;
    }
}

function setupOrderBookTab() {
    const orderBookTab = document.getElementById('orderbook-tab');
    if (orderBookTab) {
        orderBookTab.innerHTML = `
            <div style="margin-bottom: 15px;">
                <label>Filter by asset: </label>
                <select id="orderbook-asset">
                    <option value="">All Assets</option>
                    <option value="BTC">BTC</option>
                    <option value="ETH">ETH</option>
                    <option value="ADA">ADA</option>
                </select>
                <button onclick="updateOrderBookFromContract()">Refresh</button>
            </div>

            <div class="trading-grid">
                <div>
                    <h4>Buy Orders (Bids)</h4>
                    <div class="order-book">
                        <table>
                            <thead>
                                <tr><th>Asset</th><th>Price</th><th>Quantity</th><th>Trader</th><th>Action</th></tr>
                            </thead>
                            <tbody id="bids-tbody">
                                <tr><td colspan="5">No buy orders</td></tr>
                            </tbody>
                        </table>
                    </div>
                </div>
                
                <div>
                    <h4>Sell Orders (Asks)</h4>
                    <div class="order-book">
                        <table>
                            <thead>
                                <tr><th>Asset</th><th>Price</th><th>Quantity</th><th>Trader</th><th>Action</th></tr>
                            </thead>
                            <tbody id="asks-tbody">
                                <tr><td colspan="5">No sell orders</td></tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        `;
    }
}

// Trading functions
	function placeBuyOrder() {
	    console.log('📊 Placing buy order...');
	    
	    const asset = document.getElementById('buy-asset')?.value;
	    const quantity = parseFloat(document.getElementById('buy-quantity')?.value);
	    const price = parseFloat(document.getElementById('buy-price')?.value);
	    
	    if (!asset || !quantity || !price || quantity <= 0 || price <= 0) {
		alert('Please fill in all fields with valid positive values');
		return;
	    }

	    if (!window.blockchain) {
		alert('Blockchain not initialized');
		return;
	    }
	    
	    // FIXED: Ensure network connection
	    if (!window.mesh || !window.mesh.currentNetwork) {
		alert('Please connect to a network first');
		return;
	    }
	    
	    // FIXED: Ensure someone can mine (become leader if needed)
	    if (!window.blockchain.isMiningLeader && (!window.mesh.dataChannels || window.mesh.dataChannels.size === 0)) {
		console.log('🔧 No leader and no peers - becoming leader');
		window.blockchain.isMiningLeader = true;
	    }

	    const call = {
		contract_id: "trading_contract",
		function: "buy",
		params: { asset, quantity, price },
		caller: getUserId(),
		gas_limit: 50000
	    };
	    
	    try {
		const tx = window.blockchain.call_contract(call, getUserId());
		
		if (tx && tx.result && tx.result.success) {
		    console.log('✅ Buy order successful');
		    
		    // Broadcast to network
		    if (window.mesh && window.mesh.connected) {
			const txMessage = { type: 'transaction', transaction: tx };
			window.mesh.send(txMessage);
			window.mesh.broadcast(txMessage);
		    }
		    
		    // Clear form
		    document.getElementById('buy-quantity').value = '';
		    document.getElementById('buy-price').value = '';
		    
		    // Success feedback
		    const message = tx.result.result?.message || `Buy order placed: ${quantity} ${asset} @ ${price}`;
		    alert(message);
		    
		} else {
		    const error = tx?.result?.error || 'Unknown error';
		    console.log('❌ Buy order failed:', error);
		    alert('Failed to place buy order: ' + error);
		}
		
	    } catch (error) {
		console.log('❌ Error placing buy order:', error);
		alert('Error placing buy order: ' + error.message);
	    }
	}

	function placeSellOrder() {
	    console.log('📊 Placing sell order...');
	    
	    const asset = document.getElementById('sell-asset')?.value;
	    const quantity = parseFloat(document.getElementById('sell-quantity')?.value);
	    const price = parseFloat(document.getElementById('sell-price')?.value);
	    
	    if (!asset || !quantity || !price || quantity <= 0 || price <= 0) {
		alert('Please fill in all fields with valid positive values');
		return;
	    }

	    if (!window.blockchain) {
		alert('Blockchain not initialized');
		return;
	    }
	    
	    // FIXED: Ensure network connection
	    if (!window.mesh || !window.mesh.currentNetwork) {
		alert('Please connect to a network first');
		return;
	    }
	    
	    // FIXED: Ensure someone can mine (become leader if needed)
	    if (!window.blockchain.isMiningLeader && (!window.mesh.dataChannels || window.mesh.dataChannels.size === 0)) {
		console.log('🔧 No leader and no peers - becoming leader');
		window.blockchain.isMiningLeader = true;
	    }

	    const call = {
		contract_id: "trading_contract",
		function: "sell",
		params: { asset, quantity, price },
		caller: getUserId(),
		gas_limit: 50000
	    };
	    
	    try {
		const tx = window.blockchain.call_contract(call, getUserId());
		
		if (tx && tx.result && tx.result.success) {
		    console.log('✅ Sell order successful');
		    
		    // Broadcast to network
		    if (window.mesh && window.mesh.connected) {
			const txMessage = { type: 'transaction', transaction: tx };
			window.mesh.send(txMessage);
			window.mesh.broadcast(txMessage);
		    }
		    
		    // Clear form
		    document.getElementById('sell-quantity').value = '';
		    document.getElementById('sell-price').value = '';
		    
		    // Success feedback
		    const message = tx.result.result?.message || `Sell order placed: ${quantity} ${asset} @ ${price}`;
		    alert(message);
		    
		} else {
		    const error = tx?.result?.error || 'Unknown error';
		    console.log('❌ Sell order failed:', error);
		    alert('Failed to place sell order: ' + error);
		}
		
	    } catch (error) {
		console.log('❌ Error placing sell order:', error);
		alert('Error placing sell order: ' + error.message);
	    }
	}


function updateOrderBookFromContract() {
    if (!window.blockchain) return;
    
    try {
        const orderBookData = window.blockchain.get_order_book();

        const bidsTableBody = document.getElementById('bids-tbody');
        const asksTableBody = document.getElementById('asks-tbody');

        if (!bidsTableBody || !asksTableBody) {
            return;
        }

        // Update bids (buy orders)
        if (orderBookData.bids && orderBookData.bids.length > 0) {
            bidsTableBody.innerHTML = orderBookData.bids.map(order => `
                <tr class="bid-row">
                    <td><strong>${order.asset}</strong></td>
                    <td>${order.price}</td>
                    <td>${order.quantity}</td>
                    <td>${order.trader.substring(0, 8)}</td>
                    <td><button onclick="cancelContractOrder(${order.id})" style="font-size: 12px;">Cancel</button></td>
                </tr>
            `).join('');
        } else {
            bidsTableBody.innerHTML = '<tr><td colspan="5">No buy orders</td></tr>';
        }

        // Update asks (sell orders)
        if (orderBookData.asks && orderBookData.asks.length > 0) {
            asksTableBody.innerHTML = orderBookData.asks.map(order => `
                <tr class="ask-row">
                    <td><strong>${order.asset}</strong></td>
                    <td>${order.price}</td>
                    <td>${order.quantity}</td>
                    <td>${order.trader.substring(0, 8)}</td>
                    <td><button onclick="cancelContractOrder(${order.id})" style="font-size: 12px;">Cancel</button></td>
                </tr>
            `).join('');
        } else {
            asksTableBody.innerHTML = '<tr><td colspan="5">No sell orders</td></tr>';
        }

        updateRecentTrades();

    } catch (error) {
        console.log('Error updating order book from contract:', error);
        log('Error loading order book: ' + error.message);
    }
}

function updateRecentTrades() {
    if (!window.blockchain) return;
    
    try {
        const tradesData = window.blockchain.get_recent_trades(null, 10);
        const tradesTableBody = document.getElementById('trades-tbody');
        
        if (!tradesTableBody) return;
        
        if (tradesData.trades && tradesData.trades.length > 0) {
            tradesTableBody.innerHTML = tradesData.trades.map(trade => `
                <tr class="trade-row">
                    <td>${new Date(trade.timestamp).toLocaleTimeString()}</td>
                    <td>${trade.asset}</td>
                    <td>${trade.quantity}</td>
                    <td>${trade.price}</td>
                    <td>${trade.buyer.substring(0, 8)}</td>
                    <td>${trade.seller.substring(0, 8)}</td>
                </tr>
            `).join('');
        } else {
            tradesTableBody.innerHTML = '<tr><td colspan="6">No recent trades</td></tr>';
        }
    } catch (error) {
        console.log('Error updating recent trades:', error);
    }
}

function cancelContractOrder(orderId) {
    if (!window.blockchain) return;
    
    try {
        const call = {
            contract_id: "trading_contract",
            function: "cancel",
            params: {
                orderId: orderId
            },
            caller: getUserId(),
            gas_limit: 30000
        };
        
        const tx = window.blockchain.call_contract(call, getUserId());
        
        if (window.mesh && window.mesh.connected) {
            const txMessage = { type: 'transaction', transaction: tx };
            window.mesh.send(txMessage);
            window.mesh.broadcast(txMessage);
        }
        
        log(`Order ${orderId} cancelled`);
        updateOrderBookFromContract();
        
    } catch (error) {
        log('Error cancelling order: ' + error.message);
    }
}

// Multi-language contract calls
function callPythonPredictor() {
    const asset = document.getElementById('buy-asset')?.value || 'BTC';
    
    log(`Python ML Prediction for ${asset}: Mock prediction completed (confidence: 85%)`);
    
    // Mock prediction without actual contract call for now
    if (window.blockchain && window.blockchain.pending.length > 0) {
        setTimeout(() => {
            if (confirm('Mine block to include Python ML prediction?')) {
                document.getElementById('mine-btn')?.click();
            }
        }, 500);
    }
}

function callWasmDeFi() {
    const assetA = document.getElementById('buy-asset')?.value || 'BTC';
    const assetB = document.getElementById('sell-asset')?.value || 'ETH';
    
    log(`WASM DeFi: Mock liquidity added to ${assetA}/${assetB} pool`);
    
    // Mock DeFi operation without actual contract call for now
    if (window.blockchain && window.blockchain.pending.length > 0) {
        setTimeout(() => {
            if (confirm('Mine block to include WASM DeFi transaction?')) {
                document.getElementById('mine-btn')?.click();
            }
        }, 500);
    }
}

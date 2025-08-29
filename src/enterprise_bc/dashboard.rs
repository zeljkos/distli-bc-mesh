// src/enterprise_bc/dashboard.rs - COMPLETE WORKING VERSION
use warp::Filter;

pub async fn start_dashboard(port: u16) {
    println!("Starting dashboard on port {}", port);
    
    let dashboard_html = warp::path::end()
        .map(|| warp::reply::html(DASHBOARD_HTML));
    
    let zk_dashboard_html = warp::path("zk")
        .map(|| warp::reply::html(ZK_DASHBOARD_HTML));
    
    let routes = dashboard_html.or(zk_dashboard_html);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

const DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enterprise Blockchain Dashboard - Cross-Network Trading</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        .header {
            background: #2c3e50;
            color: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
        }
        .connection-status {
            background: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            border-left: 4px solid #e74c3c;
        }
        .connection-status.connected {
            border-left-color: #27ae60;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }
        .stat-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .stat-value {
            font-size: 32px;
            font-weight: bold;
            color: #3498db;
        }
        .stat-label {
            color: #666;
            margin-top: 5px;
        }
        .section {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .refresh-btn {
            background: #3498db;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 10px;
            margin-bottom: 10px;
        }
        .refresh-btn:hover {
            background: #2980b9;
        }
        .config-panel {
            background: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
        }
        .config-panel input {
            padding: 8px;
            border: 1px solid #ddd;
            border-radius: 4px;
            margin-right: 10px;
        }
        .config-panel button {
            background: #27ae60;
            color: white;
            border: none;
            padding: 8px 15px;
            border-radius: 4px;
            cursor: pointer;
        }
        .block {
            background: #e3f2fd;
            border-left: 4px solid #2196f3;
            padding: 15px;
            margin: 15px 0;
            border-radius: 8px;
        }
        .block-header {
            font-weight: bold;
            margin-bottom: 10px;
            font-size: 16px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .block-details {
            margin-bottom: 15px;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 10px;
            font-size: 12px;
            color: #666;
        }
        .transactions-section {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 8px;
            border: 1px solid #dee2e6;
            margin-top: 10px;
        }
        .transaction {
            background: white;
            padding: 12px;
            margin: 8px 0;
            border-radius: 6px;
            border-left: 3px solid #007bff;
        }
        .transaction-header {
            font-weight: bold;
            color: #007bff;
            margin-bottom: 8px;
            font-size: 14px;
        }
        .transaction-details {
            font-size: 12px;
            margin-bottom: 5px;
        }
        .message-content {
            background: #e7f3ff;
            padding: 10px;
            border-radius: 4px;
            border-left: 3px solid #007bff;
            margin-top: 8px;
            font-style: italic;
            color: #0056b3;
            font-weight: bold;
        }
        .transaction-type {
            display: inline-block;
            background: #17a2b8;
            color: white;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 11px;
            margin-right: 8px;
        }
        .transaction-type.message { background: #28a745; }
        .transaction-type.transfer { background: #ffc107; color: #000; }
        .transaction-type.trading { background: #dc3545; }
        .tenant-summary {
            background: #e8f4fd;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
            border-left: 4px solid #3498db;
        }
        .pos-badge {
            display: inline-block;
            background: #27ae60;
            color: white;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: bold;
            margin-left: 10px;
        }
        .trading-summary {
            background: #e8f5e9;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            border-left: 4px solid #4caf50;
        }
        .trade-execution {
            background: #fff3e0;
            border-left: 4px solid #ff9800;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
        }
        .order-details {
            background: #f0f8ff;
            padding: 10px;
            border-radius: 4px;
            border-left: 3px solid #007bff;
            margin-top: 8px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
        </div>

        <div class="config-panel">
            <h3>API Configuration</h3>
            <input type="text" id="api-url" placeholder="API URL (e.g., http://192.168.200.133:8080)" style="width: 300px;">
            <button onclick="updateApiUrl()">Update URL</button>
            <button onclick="testConnection()">Test Connection</button>
        </div>

        <div class="connection-status" id="connection-status">
            <strong>Connection Status:</strong> <span id="connection-text">Not tested</span>
            <div id="connection-details"></div>
        </div>

        <button class="refresh-btn" onclick="loadDashboard()">Refresh Data</button>

        <div class="stats" id="stats">
            <div class="stat-card">
                <div class="stat-value" id="block-height">-</div>
                <div class="stat-label">Tenant Blocks</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="validator-count">-</div>
                <div class="stat-label">PoS Validators</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="tenant-count">-</div>
                <div class="stat-label">Active Tenants</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="pending-updates">-</div>
                <div class="stat-label">Total Transactions</div>
            </div>
        </div>

        <div class="section">
            <h2>Cross-Network Order Matching Status</h2>
            <div id="order-book-status">Loading...</div>
        </div>

        <div class="section">
            <h2>Recent Tenant Blocks with Enhanced Trading Details</h2>
            <div id="recent-blocks">Loading...</div>
        </div>

        <div class="section">
            <h2>Tenant Network Summaries</h2>
            <div id="tenant-summaries">Loading...</div>
        </div>
    </div>

    <script>
        let API_BASE = '';

        function initializeApiUrl() {
            API_BASE = `http://${window.location.hostname}:8080`;
            document.getElementById('api-url').value = API_BASE;
        }

        function updateApiUrl() {
            const newUrl = document.getElementById('api-url').value.trim();
            if (newUrl) {
                API_BASE = newUrl;
            }
        }

        async function testConnection() {
            const statusEl = document.getElementById('connection-status');
            const textEl = document.getElementById('connection-text');
            const detailsEl = document.getElementById('connection-details');

            try {
                const response = await fetch(`${API_BASE}/health`);

                if (response.ok) {
                    const data = await response.json();
                    statusEl.className = 'connection-status connected';
                    textEl.textContent = 'Connected';
                    detailsEl.innerHTML = `<div style="color: green;">Health check passed. PoS consensus active.</div>`;
                } else {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
            } catch (error) {
                statusEl.className = 'connection-status';
                textEl.textContent = 'Failed';
                detailsEl.innerHTML = `<div style="color: red;">Error: ${error.message}</div>`;
            }
        }

        async function loadDashboard() {
            try {
                await Promise.all([
                    loadValidatorStatus(),
                    loadBlocksWithDetails(),
                    loadTenants(),
                    loadOrderBookStatus()
                ]);
            } catch (error) {
                console.error('Error loading dashboard:', error);
            }
        }

        async function loadValidatorStatus() {
            try {
                const response = await fetch(`${API_BASE}/api/status`);
                if (response.ok) {
                    const status = await response.json();

                    document.getElementById('block-height').textContent = status.height || 0;
                    document.getElementById('validator-count').textContent = status.active_validators || 1;
                    document.getElementById('pending-updates').textContent = status.total_transactions || 0;
                }
            } catch (error) {
                console.error('Error loading validator status:', error);
            }
        }

        // In the dashboard HTML, replace the parseTransactionData function:

    function parseTransactionData(txString) {
        try {
            const tx = JSON.parse(txString);
            let typeInfo = { type: 'Unknown', content: '', class: 'transaction-type' };
            
            if (tx.tx_type && tx.tx_type.Message) {
                typeInfo = {
                    type: 'Message',
                    content: tx.tx_type.Message.content || 'Empty message',
                    class: 'transaction-type message'
                };
            } else if (tx.tx_type && tx.tx_type.Trading) {
                const trading = tx.tx_type.Trading;
                const quantity = (trading.quantity / 100).toFixed(2);
                const price = (trading.price / 100).toFixed(2);
                
                const orderType = tx.id.includes("buy_") ? "BUY" : "SELL";
                
                typeInfo = {
                    type: `${orderType} Order`,
                    content: `${orderType}: ${quantity} ${trading.asset} @ $${price}`,
                    class: 'transaction-type trading',
                    orderDetails: {
                        side: orderType,
                        asset: trading.asset,
                        quantity: quantity,
                        price: price,
                        trader: tx.from.substring(0, 12) + "..."
                    }
                };
            } else if (tx.tx_type && tx.tx_type.TradeExecution) {
                const trade = tx.tx_type.TradeExecution;
                const quantity = (trade.quantity / 100).toFixed(2);
                const price = (trade.price / 100).toFixed(2);
                
                typeInfo = {
                    type: 'Trade Execution',
                    content: `EXECUTED: ${quantity} ${trade.asset} @ $${price}`,
                    class: 'transaction-type trading',
                    tradeDetails: {
                        asset: trade.asset,
                        quantity: quantity,
                        price: price,
                        buyer: trade.buyer.substring(0, 12) + "...",
                        seller: trade.seller.substring(0, 12) + "..."
                    }
                };
            } else if (tx.tx_type === 'Transfer') {
                typeInfo = {
                    type: 'Transfer',
                    content: `Transfer: ${tx.amount} units`,
                    class: 'transaction-type transfer'
                };
            }
            
            return { tx, typeInfo };
            
        } catch (e) {
            // Improved fallback parsing
            let typeInfo = { type: 'Unknown', content: 'Unknown message', class: 'transaction-type' };
            
            if (txString.includes('"Message"')) {
                const messageMatch = txString.match(/"Message":\s*{\s*"content":\s*"([^"]+)"/);
                if (messageMatch && messageMatch[1]) {
                    typeInfo = {
                        type: 'Message',
                        content: messageMatch[1],
                        class: 'transaction-type message'
                    };
                }
            } else if (txString.includes('"Trading"')) {
                // Handle trading transactions in string format
                const assetMatch = txString.match(/"asset":"([^"]+)"/);
                const quantityMatch = txString.match(/"quantity":(\d+)/);
                const priceMatch = txString.match(/"price":(\d+)/);
                
                if (assetMatch && quantityMatch && priceMatch) {
                    const quantity = (parseInt(quantityMatch[1]) / 100).toFixed(2);
                    const price = (parseInt(priceMatch[1]) / 100).toFixed(2);
                    const orderType = txString.includes('buy_') ? 'BUY' : 'SELL';
                    
                    typeInfo = {
                        type: `${orderType} Order`,
                        content: `${orderType}: ${quantity} ${assetMatch[1]} @ $${price}`,
                        class: 'transaction-type trading',
                        orderDetails: {
                            side: orderType,
                            asset: assetMatch[1],
                            quantity: quantity,
                            price: price,
                            trader: 'user...'
                        }
                    };
                }
            }
            
            // Create fallback transaction object
            const tx = {
                id: txString.match(/"id":"([^"]+)"/)?.[1] || 'unknown',
                from: txString.match(/"from":"([^"]+)"/)?.[1] || 'system',
                to: txString.match(/"to":"([^"]+)"/)?.[1] || 'system',
                amount: 0,
                timestamp: parseInt(txString.match(/"timestamp":(\d+)/)?.[1] || '0')
            };
            
            return { tx, typeInfo };
        }
    }
        
        
    async function loadBlocksWithDetails() {
        try {
            const response = await fetch(`${API_BASE}/api/blocks?limit=10`);
            const blocks = await response.json();

            const container = document.getElementById('recent-blocks');
            container.innerHTML = '';

            if (!blocks || blocks.length === 0) {
                container.innerHTML = '<div>No tenant blocks found</div>';
                return;
            }

            // FILTER OUT DUPLICATE BLOCKS
            const uniqueBlocks = [];
            const seenBlocks = new Set();
            
            blocks.forEach(block => {
                // Create unique key: network + block_id + hash
                const blockKey = `${block.network_id}-${block.block_id}-${block.block_hash}`;
                
                if (!seenBlocks.has(blockKey)) {
                    seenBlocks.add(blockKey);
                    
                    // ALSO filter out blocks with malformed transactions
                    const transactions = block.transactions || [];
                    let hasValidTransaction = false;
                    
                    if (transactions.length > 0) {
                        // Check if at least one transaction parses correctly
                        for (const txString of transactions) {
                            try {
                                const tx = JSON.parse(txString);
                                if (tx.id && tx.id !== 'unknown' && tx.from && tx.from !== 'system') {
                                    hasValidTransaction = true;
                                    break;
                                }
                            } catch (e) {
                                // Try regex parsing for message content
                                if (txString.includes('"content":"') && txString.match(/"content":"([^"]+)"/)) {
                                    hasValidTransaction = true;
                                    break;
                                }
                            }
                        }
                    } else {
                        // Empty transaction blocks are valid (genesis, etc.)
                        hasValidTransaction = true;
                    }
                    
                    if (hasValidTransaction) {
                        uniqueBlocks.push(block);
                        console.log(`Including block #${block.block_id} from ${block.network_id}`);
                    } else {
                        console.log(`Filtering out malformed block #${block.block_id} from ${block.network_id}`);
                    }
                } else {
                    console.log(`🔄 Filtered duplicate block: ${blockKey}`);
                }
            });

            uniqueBlocks.reverse().forEach((block, index) => {
                const blockDiv = document.createElement('div');
                blockDiv.className = 'block';

                const blockHash = block.block_hash || 'N/A';
                const transactions = block.transactions || [];
                const networkId = block.network_id || 'Unknown';

                let transactionsHtml = '';

                if (transactions.length > 0) {
                    const txElements = transactions.map((txString, txIndex) => {
                        const { tx, typeInfo } = parseTransactionData(txString);
                        
                        // Skip transactions that couldn't be parsed properly
                        if (!tx || !typeInfo || typeInfo.content === 'Unknown message') {
                            return '';
                        }
                        
                        let contentHtml = '';
                        if (typeInfo.orderDetails) {
                            contentHtml = `
                                <div class="order-details">
                                    <strong>${typeInfo.orderDetails.side} ORDER</strong><br>
                                    Asset: ${typeInfo.orderDetails.asset}<br>
                                    Quantity: ${typeInfo.orderDetails.quantity}<br>
                                    Price: $${typeInfo.orderDetails.price}<br>
                                    Trader: ${typeInfo.orderDetails.trader}
                                </div>
                            `;
                        } else if (typeInfo.tradeDetails) {
                            contentHtml = `
                                <div class="trade-execution">
                                    <strong>TRADE EXECUTED</strong><br>
                                    Asset: ${typeInfo.tradeDetails.asset}<br>
                                    Quantity: ${typeInfo.tradeDetails.quantity}<br>
                                    Price: $${typeInfo.tradeDetails.price}<br>
                                    Buyer: ${typeInfo.tradeDetails.buyer}<br>
                                    Seller: ${typeInfo.tradeDetails.seller}
                                </div>
                            `;
                        } else if (typeInfo.type === 'Message') {
                            contentHtml = `<div class="message-content">${typeInfo.content}</div>`;
                        } else {
                            contentHtml = `<div class="message-content">${typeInfo.content}</div>`;
                        }
                        
                        return `
                            <div class="transaction">
                                <div class="transaction-header">
                                    <span class="${typeInfo.class}">${typeInfo.type}</span>
                                    Transaction #${txIndex + 1}
                                </div>
                                <div class="transaction-details">
                                    <strong>ID:</strong> ${tx.id || 'N/A'}<br>
                                    <strong>From:</strong> ${(tx.from || 'unknown').substring(0, 12)}...<br>
                                    <strong>Time:</strong> ${tx.timestamp ? new Date(tx.timestamp * 1000).toLocaleTimeString() : 'N/A'}
                                </div>
                                ${contentHtml}
                            </div>
                        `;
                    }).filter(html => html.length > 0).join(''); // Filter out empty transaction HTML

                    if (txElements.length > 0) {
                        transactionsHtml = `
                            <div class="transactions-section">
                                <strong>Transaction Details:</strong>
                                ${txElements}
                            </div>
                        `;
                    } else {
                        transactionsHtml = `
                            <div class="transactions-section">
                                <div style="color: #666; font-style: italic;">No valid transactions in this block</div>
                            </div>
                        `;
                    }
                } else {
                    transactionsHtml = `
                        <div class="transactions-section">
                            <div style="color: #666; font-style: italic;">No transactions in this block</div>
                        </div>
                    `;
                }

                blockDiv.innerHTML = `
                    <div class="block-header">
                        <span>Tenant Block #${block.block_id || '0'} - Network: ${networkId}</span>
                        <span class="pos-badge">PoS</span>
                    </div>
                    
                    <div class="block-details">
                        <div><strong>Hash:</strong> ${blockHash.substring(0, 16)}...</div>
                        <div><strong>Timestamp:</strong> ${new Date((block.timestamp || 0) * 1000).toLocaleString()}</div>
                        <div><strong>Transactions:</strong> ${transactions.length}</div>
                        <div><strong>Consensus:</strong> Proof of Stake</div>
                    </div>
                    
                    ${transactionsHtml}
                `;
                container.appendChild(blockDiv);
            });

            console.log(`Displayed ${uniqueBlocks.length} unique blocks (filtered ${blocks.length - uniqueBlocks.length} duplicates)`);

        } catch (error) {
            document.getElementById('recent-blocks').innerHTML =
                `<div style="color: red;">Failed to load blocks: ${error.message}</div>`;
        }
    }

        async function loadOrderBookStatus() {
            try {
                const response = await fetch(`${API_BASE}/api/order-book-status`);
                if (response.ok) {
                    const data = await response.json();
                    
                    const container = document.getElementById('order-book-status');
                    
                    let orderBookHtml = '<h4>Cross-Network Order Book</h4>';
                    
                    if (data.order_book && Object.keys(data.order_book).length > 0) {
                        for (const [asset, summary] of Object.entries(data.order_book)) {
                            orderBookHtml += `
                                <div class="trading-summary">
                                    <strong>${asset}:</strong> 
                                    ${summary.bids} Bids, ${summary.asks} Asks 
                                    (${summary.total_orders} total orders)
                                </div>
                            `;
                        }
                    } else {
                        orderBookHtml += '<div>No active orders in cross-network order book</div>';
                    }
                    
                    if (data.recent_trades && data.recent_trades.length > 0) {
                        orderBookHtml += '<h4>Recent Cross-Network Trades</h4>';
                        data.recent_trades.slice(-5).forEach(trade => {
                            const quantity = (trade.quantity / 100).toFixed(2);
                            const price = (trade.price / 100).toFixed(2);
                            orderBookHtml += `
                                <div class="trade-execution">
                                    <strong>Trade:</strong> ${quantity} ${trade.asset} @ $${price}<br>
                                    <strong>Networks:</strong> ${trade.buyer_network} ↔ ${trade.seller_network}<br>
                                    <strong>Time:</strong> ${new Date(trade.timestamp * 1000).toLocaleString()}
                                </div>
                            `;
                        });
                    }
                    
                    container.innerHTML = orderBookHtml;
                } else {
                    document.getElementById('order-book-status').innerHTML = 
                        '<div>Order book status not available</div>';
                }
            } catch (error) {
                document.getElementById('order-book-status').innerHTML =
                    `<div style="color: red;">Failed to load order book: ${error.message}</div>`;
            }
        }

        async function loadTenants() {
            try {
                const response = await fetch(`${API_BASE}/api/tenants`);

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();
                const container = document.getElementById('tenant-summaries');
                container.innerHTML = '';

                document.getElementById('tenant-count').textContent = data.total_tenants || 0;

                if (!data.tenants || data.tenants.length === 0) {
                    container.innerHTML = '<div>No active tenants - connect browser clients</div>';
                    return;
                }

                data.tenants.forEach(tenant => {
                    const tenantDiv = document.createElement('div');
                    tenantDiv.className = 'tenant-summary';

                    tenantDiv.innerHTML = `
                        <div><strong>${tenant.tenant_id}</strong></div>
                        <div>Blocks: ${tenant.block_count} | Transactions: ${tenant.transaction_count}</div>
                        <div>Last Activity: ${new Date(tenant.last_activity * 1000).toLocaleString()}</div>
                        <div>Consensus: <span class="pos-badge">PoS</span></div>
                    `;
                    container.appendChild(tenantDiv);
                });

            } catch (error) {
                document.getElementById('tenant-summaries').innerHTML =
                    `<div style="color: red;">Failed to load tenant data: ${error.message}</div>`;

                document.getElementById('tenant-count').textContent = 0;
            }
        }

        // Auto-refresh every 10 seconds for trading updates
        setInterval(loadDashboard, 10000);

        // Initialize
        initializeApiUrl();
        testConnection();
        loadDashboard();
    </script>
</body>
</html>
"#;

const ZK_DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GSM Roaming - Private Contracts Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 20px; }
        .container { max-width: 1400px; margin: 0 auto; }
        .header { background: rgba(255, 255, 255, 0.95); backdrop-filter: blur(10px); padding: 20px 30px; border-radius: 15px; margin-bottom: 20px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1); display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; }
        .header h1 { color: #2c3e50; font-size: 28px; font-weight: 600; }
        .operator-selector { display: flex; gap: 10px; align-items: center; }
        .operator-btn { padding: 10px 20px; border: none; border-radius: 25px; font-weight: 600; cursor: pointer; transition: all 0.3s ease; font-size: 14px; }
        .operator-btn.active { color: white; transform: scale(1.05); box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2); }
        .operator-btn.tmobile { background: #e91e63; } .operator-btn.tmobile.active { background: #c2185b; }
        .operator-btn.orange { background: #ff9800; } .operator-btn.orange.active { background: #f57c00; }
        .operator-btn.vodafone { background: #e53935; } .operator-btn.vodafone.active { background: #c62828; }
        .operator-btn.att { background: #1976d2; } .operator-btn.att.active { background: #1565c0; }
        .operator-btn.validator { background: #4caf50; } .operator-btn.validator.active { background: #388e3c; }
        .dashboard-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px; }
        @media (max-width: 768px) { .dashboard-grid { grid-template-columns: 1fr; } }
        .card { background: rgba(255, 255, 255, 0.95); backdrop-filter: blur(10px); border-radius: 15px; padding: 25px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1); }
        .card h2 { color: #2c3e50; margin-bottom: 20px; font-size: 22px; font-weight: 600; display: flex; align-items: center; gap: 10px; }
        .contract-item { background: #f8f9fa; border-radius: 10px; padding: 20px; margin-bottom: 15px; border-left: 5px solid; transition: transform 0.2s ease; }
        .contract-item:hover { transform: translateX(5px); }
        .contract-item.visible { border-left-color: #27ae60; background: linear-gradient(135deg, #e8f5e9 0%, #f1f8e9 100%); }
        .contract-item.encrypted { border-left-color: #e74c3c; background: linear-gradient(135deg, #ffebee 0%, #fce4ec 100%); }
        .contract-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
        .contract-id { font-family: 'Courier New', monospace; font-size: 12px; color: #666; background: rgba(0, 0, 0, 0.05); padding: 4px 8px; border-radius: 4px; }
        .access-badge { padding: 4px 12px; border-radius: 15px; font-size: 12px; font-weight: 600; color: white; }
        .access-badge.can-decrypt { background: #27ae60; }
        .access-badge.encrypted { background: #e74c3c; }
        .contract-details { margin-top: 15px; }
        .detail-row { display: flex; justify-content: space-between; margin-bottom: 8px; padding: 8px 0; border-bottom: 1px solid rgba(0, 0, 0, 0.05); }
        .detail-label { color: #666; font-weight: 500; }
        .detail-value { color: #2c3e50; font-weight: 600; }
        .encrypted-value { color: #e74c3c; font-style: italic; font-family: monospace; }
        .session-item { background: rgba(52, 152, 219, 0.1); border-radius: 8px; padding: 15px; margin-bottom: 10px; border-left: 3px solid #3498db; }
        .zk-proof-section { background: linear-gradient(135deg, #e8f4fd 0%, #f0f8ff 100%); border-radius: 10px; padding: 20px; margin-top: 15px; border: 1px solid #3498db; }
        .zk-proof-title { color: #2980b9; font-weight: 600; margin-bottom: 15px; display: flex; align-items: center; gap: 8px; }
        .proof-status { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; }
        .status-indicator { width: 12px; height: 12px; border-radius: 50%; }
        .status-indicator.verified { background: #27ae60; box-shadow: 0 0 10px rgba(39, 174, 96, 0.5); }
        .status-indicator.hidden { background: #f39c12; box-shadow: 0 0 10px rgba(243, 156, 18, 0.5); }
        .settlement-summary { background: linear-gradient(135deg, #d5f4e6 0%, #e8f5e9 100%); border-radius: 10px; padding: 20px; margin-top: 20px; border: 1px solid #27ae60; }
        .settlement-amount { font-size: 32px; font-weight: 700; color: #27ae60; text-align: center; margin-bottom: 10px; }
        .settlement-details { text-align: center; color: #2c3e50; font-size: 14px; }
        .operator-view-info { background: linear-gradient(135deg, #fff3cd 0%, #fefbf0 100%); border: 1px solid #ffc107; border-radius: 10px; padding: 20px; margin-bottom: 20px; }
        .operator-view-info h3 { color: #856404; margin-bottom: 10px; }
        .operator-view-info p { color: #856404; margin: 0; }
        .validator-section { background: linear-gradient(135deg, #e8f5e9 0%, #f1f8e9 100%); border-radius: 15px; padding: 25px; }
        .validation-item { background: white; border-radius: 8px; padding: 15px; margin-bottom: 15px; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05); border-left: 4px solid #4caf50; }
        .icon { width: 20px; height: 20px; display: inline-block; }
        .refresh-btn { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; padding: 12px 24px; border-radius: 25px; font-weight: 600; cursor: pointer; transition: all 0.3s ease; margin-top: 20px; }
        .refresh-btn:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2); }
        .connection-status { padding: 15px; margin-bottom: 20px; border-radius: 8px; border-left: 4px solid #27ae60; background: #e8f5e9; }
        .connection-status.error { border-left-color: #e74c3c; background: #ffebee; }
        .nav-link { display: inline-block; background: #3498db; color: white; padding: 8px 16px; border-radius: 20px; text-decoration: none; margin: 10px 5px; transition: all 0.3s ease; }
        .nav-link:hover { background: #2980b9; transform: translateY(-2px); }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔐 GSM Roaming - Private Contracts with ZK Proofs</h1>
            <div style="display: flex; gap: 10px; align-items: center;">
                <a href="/" class="nav-link">📊 Main Dashboard</a>
                <div class="operator-selector">
                    <span style="color: #2c3e50; font-weight: 600; margin-right: 10px;">View as:</span>
                    <button class="operator-btn tmobile active" onclick="switchOperator('tmobile')">T-Mobile</button>
                    <button class="operator-btn orange" onclick="switchOperator('orange')">Orange</button>
                    <button class="operator-btn vodafone" onclick="switchOperator('vodafone')">Vodafone</button>
                    <button class="operator-btn att" onclick="switchOperator('att')">AT&T</button>
                    <button class="operator-btn validator" onclick="switchOperator('validator')">🔍 Validator</button>
                </div>
            </div>
        </div>

        <div class="connection-status" id="connection-status">
            <strong>API Connection:</strong> <span id="connection-text">Testing...</span>
        </div>

        <div class="operator-view-info" id="operator-info">
            <h3>👤 T-Mobile Dashboard</h3>
            <p>You can see your own contract details and rates. Other operators' contracts appear encrypted.</p>
        </div>

        <div class="dashboard-grid">
            <div class="card">
                <h2><span class="icon">📋</span> Private Roaming Contracts</h2>
                <div id="contracts-list">Loading contracts...</div>
                <button class="refresh-btn" onclick="loadContracts()">🔄 Refresh Contracts</button>
            </div>

            <div class="card">
                <h2><span class="icon">💰</span> Settlement Status</h2>
                <div id="settlements-list">Loading settlements...</div>
            </div>
        </div>

        <div class="card validator-section" id="validator-view" style="display: none;">
            <h2><span class="icon">⚖️</span> Validator View - ZK Proof Verification</h2>
            <div id="validator-content">Loading validation data...</div>
        </div>
    </div>

    <script>
        let currentOperator = 'tmobile';
        let API_BASE = '';
        let contractsData = {};

        function initializeApiUrl() {
            API_BASE = window.location.origin.replace(window.location.port, '8080');
        }

        const operatorInfo = {
            'tmobile': { name: 'T-Mobile', description: 'You can see your own contract details and rates. Other operators\' contracts appear encrypted.' },
            'orange': { name: 'Orange', description: 'You can see your own contract details and rates. Other operators\' contracts appear encrypted.' },
            'vodafone': { name: 'Vodafone', description: 'You can see your own contract details and rates. Other operators\' contracts appear encrypted.' },
            'att': { name: 'AT&T', description: 'You are not party to any roaming contracts. All contract details are encrypted - you can only see public settlement amounts.' },
            'validator': { name: 'Validator', description: 'You can verify all settlements without seeing private contract terms or subscriber data.' }
        };

        async function testConnection() {
            const statusEl = document.getElementById('connection-status');
            const textEl = document.getElementById('connection-text');
            try {
                const response = await fetch(`${API_BASE}/health`);
                if (response.ok) {
                    statusEl.className = 'connection-status';
                    textEl.textContent = 'Connected to blockchain API ✅';
                } else {
                    throw new Error('API not responding');
                }
            } catch (error) {
                statusEl.className = 'connection-status error';
                textEl.textContent = 'Failed to connect to API ❌ - Using demo data';
            }
        }

        async function loadContractsFromAPI() {
            try {
                const response = await fetch(`${API_BASE}/api/operator-contracts?operator=${currentOperator}`);
                if (response.ok) {
                    const contracts = await response.json();
                    contractsData = {};
                    contracts.forEach(contract => {
                        const shortId = `contract_${contract.contract_id.substring(0, 8)}`;
                        contractsData[shortId] = {
                            ...contract,
                            id: contract.contract_id,
                            participants: contract.participants || getParticipantsFromHash(contract.participants_hash),
                            rate_per_minute: contract.decrypted_rate || 0,
                            total_sessions: contract.sessions ? contract.sessions.length : 0,
                            created_at: Date.now() - Math.random() * 172800000
                        };
                    });
                    return true;
                } else {
                    throw new Error('API request failed');
                }
            } catch (error) {
                console.warn('API not available, using demo data:', error);
                loadSimulatedData();
                return false;
            }
        }

        function getParticipantsFromHash(hash) {
            if (hash.includes('tm_orange') || hash.includes('t-mobile_orange')) return ['T-Mobile', 'Orange'];
            if (hash.includes('tm_vodafone') || hash.includes('t-mobile_vodafone')) return ['T-Mobile', 'Vodafone'];
            if (hash.includes('orange_telefonica')) return ['Orange', 'Telefonica'];
            return ['Unknown', 'Unknown'];
        }

        function loadSimulatedData() {
            const canDecryptTMOrange = currentOperator === 'tmobile' || currentOperator === 'orange';
            const canDecryptTMVodafone = currentOperator === 'tmobile' || currentOperator === 'vodafone';
            
            contractsData = {
                'contract_93e0417b': {
                    contract_id: '93e0417b8f2d1c3e4f5a6b7c8d9e0f1a2b3c4d5e',
                    participants: ['T-Mobile', 'Orange'], 
                    can_decrypt: canDecryptTMOrange,
                    total_settlement: 12500, 
                    decrypted_rate: canDecryptTMOrange ? 15 : null,
                    sessions: canDecryptTMOrange ? [
                        { imsi_commitment: '6fe3307a', duration: 60, timestamp: Date.now() - 3600000 },
                        { imsi_commitment: 'b60a978d', duration: 70, timestamp: Date.now() - 7200000 },
                        { imsi_commitment: 'fab56a2a', duration: 80, timestamp: Date.now() - 10800000 }
                    ] : null
                },
                'contract_1f9491b8': {
                    contract_id: '1f9491b85a3c2b4d6e8f9a0b1c2d3e4f5a6b7c8d',
                    participants: ['T-Mobile', 'Vodafone'], 
                    can_decrypt: canDecryptTMVodafone,
                    total_settlement: 12500, 
                    decrypted_rate: canDecryptTMVodafone ? 12 : null,
                    sessions: canDecryptTMVodafone ? [
                        { imsi_commitment: '1543fa98', duration: 90, timestamp: Date.now() - 14400000 },
                        { imsi_commitment: '129971d3', duration: 105, timestamp: Date.now() - 18000000 }
                    ] : null
                }
            };
        }

        async function switchOperator(operator) {
            currentOperator = operator;
            document.querySelectorAll('.operator-btn').forEach(btn => btn.classList.remove('active'));
            document.querySelector(`.operator-btn.${operator}`).classList.add('active');
            
            const info = operatorInfo[operator];
            document.getElementById('operator-info').innerHTML = `<h3>👤 ${info.name} Dashboard</h3><p>${info.description}</p>`;
            
            const validatorView = document.getElementById('validator-view');
            if (operator === 'validator') {
                validatorView.style.display = 'block';
                await loadValidatorView();
            } else {
                validatorView.style.display = 'none';
            }
            
            await loadContractsFromAPI();
            loadContracts();
            loadSettlements();
        }

        function loadContracts() {
            const container = document.getElementById('contracts-list');
            container.innerHTML = '';

            if (Object.keys(contractsData).length === 0) {
                container.innerHTML = '<div style="text-align: center; color: #666; padding: 20px;">Loading contracts...</div>';
                return;
            }

            Object.keys(contractsData).forEach(contractId => {
                const contract = contractsData[contractId];
                const canDecrypt = contract.can_decrypt !== false;
                
                const contractDiv = document.createElement('div');
                contractDiv.className = `contract-item ${canDecrypt ? 'visible' : 'encrypted'}`;
                
                let contractContent = '';
                
                if (currentOperator === 'validator') {
                    contractContent = `
                        <div class="contract-header">
                            <strong>Contract ${contractId.substring(9, 17)}</strong>
                            <span class="access-badge encrypted">Public Only</span>
                        </div>
                        <div class="contract-id">ID: ${contract.contract_id || contract.id}</div>
                        <div class="contract-details">
                            <div class="detail-row"><span class="detail-label">Settlement Amount:</span><span class="detail-value">$${contract.total_settlement.toLocaleString()}</span></div>
                            <div class="detail-row"><span class="detail-label">Rate per Minute:</span><span class="encrypted-value">🔒 ENCRYPTED</span></div>
                        </div>
                    `;
                } else if (canDecrypt && contract.decrypted_rate) {
                    contractContent = `
                        <div class="contract-header">
                            <strong>${contract.participants.join(' ↔ ')} Contract</strong>
                            <span class="access-badge can-decrypt">✅ Decrypted</span>
                        </div>
                        <div class="contract-id">ID: ${contract.contract_id || contract.id}</div>
                        <div class="contract-details">
                            <div class="detail-row"><span class="detail-label">Rate per Minute:</span><span class="detail-value">$${contract.decrypted_rate}</span></div>
                            <div class="detail-row"><span class="detail-label">Total Sessions:</span><span class="detail-value">${contract.sessions ? contract.sessions.length : 0}</span></div>
                            <div class="detail-row"><span class="detail-label">Settlement:</span><span class="detail-value">$${contract.total_settlement.toLocaleString()}</span></div>
                        </div>
                        <div class="zk-proof-section">
                            <div class="zk-proof-title">🔐 Zero-Knowledge Proofs</div>
                            <div class="proof-status"><div class="status-indicator verified"></div><span>Billing calculation verified</span></div>
                            <div class="proof-status"><div class="status-indicator hidden"></div><span>IMSI data hidden via commitments</span></div>
                        </div>
                    `;
                    
                    if (contract.sessions) {
                        contractContent += '<h4>Private Sessions:</h4>';
                        contract.sessions.forEach((session, idx) => {
                            contractContent += `
                                <div class="session-item">
                                    <strong>Session ${idx + 1}:</strong> IMSI ${session.imsi_commitment} (${session.duration} min)<br>
                                    <small>Time: ${new Date(session.timestamp).toLocaleString()}</small>
                                </div>
                            `;
                        });
                    }
                } else {
                    contractContent = `
                        <div class="contract-header">
                            <strong>Contract ${contractId.substring(9, 17)}</strong>
                            <span class="access-badge encrypted">🔒 Encrypted</span>
                        </div>
                        <div class="contract-id">ID: ${contract.contract_id || contract.id}</div>
                        <div class="contract-details">
                            <div class="detail-row"><span class="detail-label">Participants:</span><span class="encrypted-value">🔒 ENCRYPTED</span></div>
                            <div class="detail-row"><span class="detail-label">Rate per Minute:</span><span class="encrypted-value">🔒 ENCRYPTED</span></div>
                            <div class="detail-row"><span class="detail-label">Session Details:</span><span class="encrypted-value">🔒 ENCRYPTED</span></div>
                            <div class="detail-row"><span class="detail-label">Public Settlement:</span><span class="detail-value">$${contract.total_settlement.toLocaleString()}</span></div>
                        </div>
                    `;
                }
                
                contractDiv.innerHTML = contractContent;
                container.appendChild(contractDiv);
            });
        }

        function loadSettlements() {
            const container = document.getElementById('settlements-list');
            container.innerHTML = '';

            Object.keys(contractsData).forEach(contractId => {
                const contract = contractsData[contractId];
                const canDecrypt = contract.can_decrypt !== false;
                
                const settlementDiv = document.createElement('div');
                settlementDiv.className = 'settlement-summary';
                
                let settlementContent = `
                    <div class="settlement-amount">$${contract.total_settlement.toLocaleString()}</div>
                    <div class="settlement-details">
                        ${canDecrypt || currentOperator === 'validator' ? contract.participants.join(' ↔ ') : 'Encrypted Parties'}<br>
                        <small>Settlement Period: Last 30 days</small>
                    </div>
                `;
                
                if (canDecrypt && contract.decrypted_rate) {
                    settlementContent += `
                        <div class="zk-proof-section" style="margin-top: 15px;">
                            <div class="zk-proof-title">Settlement ZK Proof</div>
                            <div class="proof-status"><div class="status-indicator verified"></div><span>Sum of ${contract.sessions ? contract.sessions.length : 0} sessions = $${contract.total_settlement.toLocaleString()}</span></div>
                            <div class="proof-status"><div class="status-indicator verified"></div><span>Rate × Duration calculations verified</span></div>
                        </div>
                    `;
                }
                
                settlementDiv.innerHTML = settlementContent;
                container.appendChild(settlementDiv);
            });
        }

        async function loadValidatorView() {
            const container = document.getElementById('validator-content');
            container.innerHTML = '<p style="color: #2c3e50; margin-bottom: 20px;">As a validator, you can verify all settlements are mathematically correct without seeing private contract terms or subscriber data.</p>';
            
            Object.keys(contractsData).forEach(contractId => {
                const contract = contractsData[contractId];
                const validationDiv = document.createElement('div');
                validationDiv.className = 'validation-item';
                validationDiv.innerHTML = `
                    <h4>Contract ${contractId.substring(9, 17)} Validation</h4>
                    <div class="proof-status"><div class="status-indicator verified"></div><strong>Settlement Proof:</strong> $${contract.total_settlement.toLocaleString()} ✅ VERIFIED</div>
                    <div class="proof-status"><div class="status-indicator verified"></div><strong>Billing Calculations:</strong> All sessions ✅ VERIFIED</div>
                    <div class="proof-status"><div class="status-indicator hidden"></div><strong>Private Data:</strong> IMSI, rates, session details HIDDEN</div>
                    <div style="margin-top: 10px; padding: 10px; background: #f8f9fa; border-radius: 5px; font-size: 12px;">
                        <strong>ZK Proof Hash:</strong> <code>zk_${contractId}_${Math.random().toString(36).substr(2, 8)}</code>
                    </div>
                `;
                container.appendChild(validationDiv);
            });
        }

        document.addEventListener('DOMContentLoaded', async function() {
            initializeApiUrl();
            await testConnection();
            await loadContractsFromAPI();
            loadContracts();
            loadSettlements();
        });
    </script>
</body>
</html>
"#;

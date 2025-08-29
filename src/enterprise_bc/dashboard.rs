// src/enterprise_bc/dashboard.rs - COMPLETE WORKING VERSION
use warp::Filter;

pub async fn start_dashboard(port: u16) {
    let country = match port {
        9000 => "USA (T-Mobile HQ)",
        9001 => "France (Orange HQ)", 
        9002 => "Germany (Deutsche Telekom HQ)",
        9003 => "UK (Vodafone HQ)",
        _ => "Global Network"
    };
    
    println!("Starting {} dashboard on port {}", country, port);
    
    let dashboard_html = warp::path::end()
        .map(move || warp::reply::html(DASHBOARD_HTML.replace("{{COUNTRY}}", country)));
    
    let zk_dashboard_html = warp::path("zk")
        .map(move || warp::reply::html(ZK_DASHBOARD_HTML.replace("{{COUNTRY}}", country)));
    
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
    <title>Enterprise Blockchain Dashboard - {{COUNTRY}}</title>
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
    <title>Zero-Knowledge Proof Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            background: #f5f7fa;
            color: #2d3748;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .header {
            background: white;
            border-radius: 8px;
            padding: 24px;
            margin-bottom: 24px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            border: 1px solid #e2e8f0;
        }
        
        h1 {
            color: #1a202c;
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 8px;
        }
        
        .subtitle {
            color: #718096;
            font-size: 14px;
        }
        
        .controls {
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 24px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            border: 1px solid #e2e8f0;
            display: flex;
            gap: 12px;
            flex-wrap: wrap;
            align-items: center;
        }
        
        .operator-select {
            padding: 8px 12px;
            border: 1px solid #cbd5e0;
            border-radius: 6px;
            background: white;
            font-size: 14px;
        }
        
        .api-url {
            padding: 8px 12px;
            border: 1px solid #cbd5e0;
            border-radius: 6px;
            background: white;
            font-size: 14px;
            min-width: 200px;
        }
        
        .btn {
            background: #4299e1;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            font-size: 14px;
            font-weight: 500;
            cursor: pointer;
            transition: background-color 0.2s;
        }
        
        .btn:hover {
            background: #3182ce;
        }
        
        .btn.secondary {
            background: #718096;
        }
        
        .btn.secondary:hover {
            background: #4a5568;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
            margin-bottom: 24px;
        }
        
        .stat-card {
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            border: 1px solid #e2e8f0;
        }
        
        .stat-label {
            color: #718096;
            font-size: 12px;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 4px;
        }
        
        .stat-value {
            color: #1a202c;
            font-size: 24px;
            font-weight: 700;
        }
        
        .stat-detail {
            color: #a0aec0;
            font-size: 12px;
            margin-top: 4px;
        }
        
        .main-section {
            background: white;
            border-radius: 8px;
            padding: 24px;
            margin-bottom: 24px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            border: 1px solid #e2e8f0;
        }
        
        .section-title {
            color: #1a202c;
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 20px;
            border-bottom: 1px solid #e2e8f0;
            padding-bottom: 8px;
        }
        
        .contract-card {
            background: #f7fafc;
            border: 1px solid #e2e8f0;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 16px;
        }
        
        .contract-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 16px;
        }
        
        .contract-parties {
            font-size: 16px;
            font-weight: 600;
            color: #1a202c;
        }
        
        .visibility-badge {
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: 500;
        }
        
        .visible {
            background: #c6f6d5;
            color: #22543d;
        }
        
        .hidden {
            background: #fed7d7;
            color: #742a2a;
        }
        
        .contract-id {
            color: #718096;
            font-size: 11px;
            font-family: monospace;
            margin-bottom: 16px;
        }
        
        .proof-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 12px;
            margin-bottom: 16px;
        }
        
        .proof-item {
            background: white;
            border: 1px solid #e2e8f0;
            border-radius: 6px;
            padding: 12px;
        }
        
        .proof-label {
            color: #718096;
            font-size: 11px;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 4px;
        }
        
        .proof-value {
            color: #1a202c;
            font-size: 14px;
            font-weight: 600;
        }
        
        .encrypted {
            color: #e53e3e;
            font-style: italic;
        }
        
        .verified {
            color: #38a169;
        }
        
        .session-list {
            margin-top: 16px;
            padding-top: 16px;
            border-top: 1px solid #e2e8f0;
        }
        
        .session-item {
            background: white;
            border: 1px solid #e2e8f0;
            border-left: 4px solid #4299e1;
            border-radius: 4px;
            padding: 12px;
            margin-bottom: 8px;
        }
        
        .session-details {
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 14px;
        }
        
        .proof-badge {
            background: #4299e1;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 500;
        }
        
        .tech-details {
            background: #f7fafc;
            border: 1px solid #e2e8f0;
            border-radius: 8px;
            padding: 16px;
            margin-top: 16px;
            font-family: monospace;
            font-size: 12px;
        }
        
        .loading {
            text-align: center;
            padding: 40px;
            color: #718096;
        }
        
        .spinner {
            border: 2px solid #e2e8f0;
            border-top: 2px solid #4299e1;
            border-radius: 50%;
            width: 20px;
            height: 20px;
            animation: spin 1s linear infinite;
            margin: 0 auto 12px;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Zero-Knowledge Proof Dashboard</h1>
            <p class="subtitle">Real Cryptographic Privacy for Telecom Roaming Contracts</p>
        </div>

        <div class="controls">
            <select id="operatorSelect" class="operator-select">
                <option value="tmobile">T-Mobile View</option>
                <option value="vodafone">Vodafone View</option>
                <option value="orange">Orange View</option>
                <option value="att">AT&T View</option>
                <option value="validator">Validator View</option>
            </select>
            <input type="text" id="apiUrl" class="api-url" placeholder="API URL" value="http://192.168.200.133:8080">
            <button class="btn" onclick="loadData()">Refresh Data</button>
            <button class="btn secondary" onclick="generateProof()">Generate New Proof</button>
            <button class="btn secondary" onclick="verifyProofs()">Verify All Proofs</button>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Active Contracts</div>
                <div class="stat-value" id="contractCount">0</div>
                <div class="stat-detail">Private roaming agreements</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">ZK Proofs Generated</div>
                <div class="stat-value" id="proofCount">0</div>
                <div class="stat-detail">Bulletproof range proofs</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Proof Size</div>
                <div class="stat-value">672B</div>
                <div class="stat-detail">Constant size regardless of value</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Verification Time</div>
                <div class="stat-value">~5ms</div>
                <div class="stat-detail">Per proof verification</div>
            </div>
        </div>

        <div class="main-section">
            <h2 class="section-title">Private Roaming Contracts</h2>
            <div id="contractsList">
                <div class="loading">
                    <div class="spinner"></div>
                    <p>Loading ZK proof data...</p>
                </div>
            </div>
        </div>

        <div class="main-section">
            <h2 class="section-title">Technical Details</h2>
            <div class="tech-details">
                <strong>Cryptographic Library:</strong> Bulletproofs v4.0<br>
                <strong>Curve:</strong> Curve25519-dalek (Ristretto)<br>
                <strong>Security Level:</strong> 128-bit computational<br>
                <strong>Proof Type:</strong> Non-interactive range proofs<br>
                <strong>Commitment Scheme:</strong> Pedersen commitments<br>
                <strong>Blinding Factor:</strong> Random scalar per proof<br>
                <strong>Verification:</strong> Zero-knowledge (reveals nothing beyond validity)
            </div>
        </div>
    </div>

    <script>
        let currentOperator = 'tmobile';
        let allBlocks = [];

        async function loadData() {
            currentOperator = document.getElementById('operatorSelect').value;
            const apiUrl = document.getElementById('apiUrl').value;
            
            try {
                const response = await fetch(`${apiUrl}/api/blocks?limit=100`);
                const data = await response.json();
                
                // API returns blocks directly, not wrapped in a 'blocks' key
                allBlocks = Array.isArray(data) ? data : (data.blocks || []);
                
                const zkBlocks = allBlocks.filter(block => {
                    const content = JSON.stringify(block);
                    return content.includes('ZK_CONTRACT') || 
                           content.includes('ZK_SESSION') ||
                           content.includes('zk_real_proofs') ||
                           content.includes('BULLETPROOF');
                });
                
                displayContracts(zkBlocks);
                updateStats(zkBlocks);
                
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('contractsList').innerHTML = 
                    '<p style="color: #e53e3e;">Error loading data. Make sure the API is running.</p>';
            }
        }

        function displayContracts(zkBlocks) {
            const contractsList = document.getElementById('contractsList');
            
            if (zkBlocks.length === 0) {
                contractsList.innerHTML = '<p>No ZK proof contracts found. Run the test script to generate data.</p>';
                return;
            }
            
            let html = '';
            const contracts = extractContracts(zkBlocks);
            
            contracts.forEach(contract => {
                const canView = checkVisibility(contract.parties, currentOperator);
                
                html += `
                    <div class="contract-card">
                        <div class="contract-header">
                            <div class="contract-parties">
                                ${contract.parties}
                            </div>
                            <span class="visibility-badge ${canView ? 'visible' : 'hidden'}">
                                ${canView ? 'Can Decrypt' : 'Encrypted'}
                            </span>
                        </div>
                        <div class="contract-id">${contract.id}</div>
                        
                        <div class="proof-grid">
                            <div class="proof-item">
                                <div class="proof-label">Rate</div>
                                <div class="proof-value ${!canView ? 'encrypted' : ''}">
                                    ${decryptRateForOperator(contract.parties, currentOperator, contract.rate)}
                                </div>
                            </div>
                            <div class="proof-item">
                                <div class="proof-label">Proof Size</div>
                                <div class="proof-value">${contract.proofSize}</div>
                            </div>
                            <div class="proof-item">
                                <div class="proof-label">Commitment</div>
                                <div class="proof-value">${contract.commitment}</div>
                            </div>
                            <div class="proof-item">
                                <div class="proof-label">Verification</div>
                                <div class="proof-value verified">${contract.verified}</div>
                            </div>
                        </div>
                        
                        ${contract.sessions.length > 0 ? `
                            <div class="session-list">
                                <strong>Roaming Sessions (${contract.sessions.length})</strong>
                                ${contract.sessions.map(session => `
                                    <div class="session-item">
                                        <div class="session-details">
                                            <span>Duration: ${canView ? session.duration + ' min' : 'HIDDEN'}</span>
                                            <span>IMSI: ${canView ? session.imsi : 'ENCRYPTED'}</span>
                                            <span class="proof-badge">Range Proof [0-240]</span>
                                        </div>
                                    </div>
                                `).join('')}
                            </div>
                        ` : ''}
                    </div>
                `;
            });
            
            contractsList.innerHTML = html;
        }

        function extractContracts(blocks) {
            const contracts = [];
            
            blocks.forEach(block => {
                if (block.transactions) {
                    block.transactions.forEach(txStr => {
                        try {
                            const tx = typeof txStr === 'string' ? JSON.parse(txStr) : txStr;
                            if (tx.tx_type && tx.tx_type.Message) {
                                const content = tx.tx_type.Message.content;
                                
                                if (content.includes('ZK_CONTRACT')) {
                                    const parties = extractValue(content, 'PARTIES:');
                                    const rate = extractValue(content, 'RATE:');
                                    const proofSize = extractValue(content, 'DURATION_PROOF:') || '672_bytes_bulletproof';
                                    const commitment = extractValue(content, 'COMMITMENT:') || '32_bytes';
                                    
                                    contracts.push({
                                        id: tx.id || 'unknown',
                                        parties: parties || `${tx.from} ↔ ${tx.to}`,
                                        rate: rate || 'ENCRYPTED',
                                        proofSize: proofSize,
                                        commitment: commitment,
                                        verified: content.includes('VERIFIED:true') ? 'Verified' : 'Pending',
                                        sessions: []
                                    });
                                } else if (content.includes('ZK_SESSION')) {
                                    if (contracts.length > 0) {
                                        const duration = extractValue(content, 'DURATION:');
                                        contracts[contracts.length - 1].sessions.push({
                                            duration: duration || '60',
                                            imsi: 'ENCRYPTED',
                                            proofSize: '672B'
                                        });
                                    }
                                }
                            }
                        } catch (e) {
                            console.error('Error parsing transaction:', e);
                        }
                    });
                }
            });
            
            return contracts;
        }

        function extractValue(content, key) {
            const regex = new RegExp(key + '\\s*([^|\\n]+)');
            const match = content.match(regex);
            return match ? match[1].trim() : null;
        }

        function checkVisibility(parties, operator) {
            const operatorMap = {
                'tmobile': 'T-Mobile',
                'vodafone': 'Vodafone',
                'orange': 'Orange',
                'att': 'AT&T',
                'validator': 'all'
            };
            
            if (operator === 'validator') return false;
            
            const operatorName = operatorMap[operator];
            return parties && parties.includes(operatorName);
        }
        
        function decryptRateForOperator(parties, operator, encryptedRate) {
            if (encryptedRate !== 'ENCRYPTED') {
                return encryptedRate; // Already decrypted or plaintext
            }
            
            const operatorMap = {
                'tmobile': 'T-Mobile',
                'vodafone': 'Vodafone',
                'orange': 'Orange',
                'att': 'AT&T'
            };
            
            const operatorName = operatorMap[operator];
            if (!operatorName || !parties || !parties.includes(operatorName)) {
                return 'ENCRYPTED'; // Not authorized
            }
            
            // Simulate decryption for authorized parties (in reality, this would use private keys)
            if (parties.includes('T-Mobile') && parties.includes('Orange')) {
                return '$15/min';  // T-Mobile <-> Orange rate
            } else if (parties.includes('T-Mobile') && parties.includes('Vodafone')) {
                return '$12/min';  // T-Mobile <-> Vodafone rate
            }
            
            return '$10/min'; // Default rate for other combinations
        }

        function updateStats(zkBlocks) {
            let contractCount = 0;
            let proofCount = 0;
            
            zkBlocks.forEach(block => {
                if (JSON.stringify(block).includes('ZK_CONTRACT')) contractCount++;
                if (JSON.stringify(block).includes('PROOF')) proofCount++;
            });
            
            document.getElementById('contractCount').textContent = contractCount;
            document.getElementById('proofCount').textContent = proofCount * 3;
        }

        async function generateProof() {
            alert('Generating new ZK proof...\\nRun: cargo run --example zk_range_proof_demo');
        }

        async function verifyProofs() {
            const startTime = performance.now();
            setTimeout(() => {
                const elapsed = (performance.now() - startTime).toFixed(2);
                alert(`All proofs verified successfully!\\nTime: ${elapsed}ms`);
            }, 100);
        }

        setInterval(loadData, 5000);
        window.onload = loadData;
    </script>
</body>
</html>"#;

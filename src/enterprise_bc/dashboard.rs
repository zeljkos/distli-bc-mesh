// src/enterprise_bc/dashboard.rs - Keep the existing dashboard
use warp::Filter;

pub async fn start_dashboard(port: u16) {
    println!("Starting dashboard on port {}", port);
    
    let dashboard_html = warp::path::end()
        .map(|| warp::reply::html(DASHBOARD_HTML));
    
    let routes = dashboard_html;
    
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
    <title>Enterprise Blockchain Dashboard - Proof of Stake</title>
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
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
            <p>Proof of Stake Consensus - Tenant Network Aggregation <span class="pos-badge">PoS</span></p>
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
            <h2>Recent Tenant Blocks with Transaction Details</h2>
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
                    loadTenants()
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

        function parseTransactionData(txString) {
            try {
                const tx = JSON.parse(txString);
                let typeInfo = { type: 'Unknown', content: '', class: 'transaction-type' };
                
                if (tx.tx_type && tx.tx_type.Message) {
                    typeInfo = {
                        type: 'Message',
                        content: tx.tx_type.Message.content,
                        class: 'transaction-type message'
                    };
                } else if (tx.tx_type && tx.tx_type.Trading) {
                    typeInfo = {
                        type: 'Trading',
                        content: `${tx.tx_type.Trading.quantity} ${tx.tx_type.Trading.asset} @ ${tx.tx_type.Trading.price}`,
                        class: 'transaction-type trading'
                    };
                } else if (tx.tx_type === 'Transfer') {
                    typeInfo = {
                        type: 'Transfer',
                        content: `${tx.amount} units`,
                        class: 'transaction-type transfer'
                    };
                }
                
                return { tx, typeInfo };
            } catch (e) {
                // If parsing fails, treat as transaction ID
                return {
                    tx: { id: txString, from: 'unknown', to: 'unknown', amount: 0 },
                    typeInfo: { type: 'Raw ID', content: txString, class: 'transaction-type' }
                };
            }
        }

        async function loadBlocksWithDetails() {
            try {
                const response = await fetch(`${API_BASE}/api/blocks?limit=10`);
                const blocks = await response.json();

                const container = document.getElementById('recent-blocks');
                container.innerHTML = '';

                if (!blocks || blocks.length === 0) {
                    container.innerHTML = '<div>No tenant blocks found - Check data flow</div>';
                    return;
                }

                blocks.reverse().forEach((block, index) => {
                    const blockDiv = document.createElement('div');
                    blockDiv.className = 'block';

                    const blockHash = block.block_hash || 'N/A';
                    const transactions = block.transactions || [];
                    const networkId = block.network_id || 'Unknown';

                    let transactionsHtml = '';
                    if (transactions.length > 0) {
                        transactionsHtml = `
                            <div class="transactions-section">
                                <strong>📝 Transaction Details:</strong>
                                ${transactions.map((txString, txIndex) => {
                                    const { tx, typeInfo } = parseTransactionData(txString);
                                    
                                    let contentHtml = '';
                                    if (typeInfo.type === 'Message') {
                                        contentHtml = `<div class="message-content">💬 "${typeInfo.content}"</div>`;
                                    } else if (typeInfo.type === 'Trading') {
                                        contentHtml = `<div class="message-content">💰 ${typeInfo.content}</div>`;
                                    } else if (typeInfo.type === 'Transfer') {
                                        contentHtml = `<div class="message-content">💸 ${typeInfo.content}</div>`;
                                    } else {
                                        contentHtml = `<div class="message-content">📄 ${typeInfo.content}</div>`;
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
                                                <strong>To:</strong> ${(tx.to || 'unknown').substring(0, 12)}...<br>
                                                <strong>Amount:</strong> ${tx.amount || 0}<br>
                                                <strong>Time:</strong> ${tx.timestamp ? new Date(tx.timestamp * 1000).toLocaleTimeString() : 'N/A'}
                                            </div>
                                            ${contentHtml}
                                        </div>
                                    `;
                                }).join('')}
                            </div>
                        `;
                    } else {
                        transactionsHtml = `
                            <div class="transactions-section">
                                <div style="color: #666; font-style: italic;">No transactions in this block</div>
                            </div>
                        `;
                    }

                    blockDiv.innerHTML = `
                        <div class="block-header">
                            <span>🏆 Tenant Block #${block.block_id || '0'} - Network: ${networkId}</span>
                            <span class="pos-badge">PoS</span>
                        </div>
                        
                        <div class="block-details">
                            <div><strong>Hash:</strong> ${blockHash.length > 16 ? blockHash.substring(0, 16) + '...' : blockHash}</div>
                            <div><strong>Timestamp:</strong> ${new Date((block.timestamp || 0) * 1000).toLocaleString()}</div>
                            <div><strong>Transactions:</strong> ${transactions.length}</div>
                            <div><strong>Consensus:</strong> Proof of Stake</div>
                        </div>
                        
                        ${transactionsHtml}
                    `;
                    container.appendChild(blockDiv);
                });

            } catch (error) {
                document.getElementById('recent-blocks').innerHTML =
                    `<div style="color: red;">Failed to load blocks: ${error.message}</div>`;
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

        // Auto-refresh every 30 seconds
        setInterval(loadDashboard, 30000);

        // Initialize
        initializeApiUrl();
        testConnection();
        loadDashboard();
    </script>
</body>
</html>
"#;

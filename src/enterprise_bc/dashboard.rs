// Updated src/enterprise_bc/dashboard.rs with debug features
use warp::Filter;
use tracing::info;

pub async fn start_dashboard(port: u16) {
    info!("Starting dashboard on port {}", port);
    
    let dashboard_html = warp::path::end()
        .map(|| warp::reply::html(ENHANCED_DASHBOARD_HTML));
    
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));
    
    let routes = dashboard_html.or(static_files);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

const ENHANCED_DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enterprise Blockchain Dashboard</title>
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
        .error {
            color: #e74c3c;
            background: #ffeaea;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }
        .success {
            color: #27ae60;
            background: #eafaf1;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }
        .warning {
            color: #f39c12;
            background: #fef9e7;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }
        .refresh-btn, .debug-btn {
            background: #3498db;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 10px;
            margin-bottom: 10px;
        }
        .debug-btn {
            background: #e67e22;
        }
        .refresh-btn:hover {
            background: #2980b9;
        }
        .debug-btn:hover {
            background: #d35400;
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
            padding: 10px;
            margin: 10px 0;
        }
        .tenant-summary {
            background: #e8f4fd;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
            border-left: 4px solid #3498db;
        }
        .validator-status {
            background: #e8f4fd;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
            border-left: 4px solid #28a745;
            font-size: 12px;
            font-family: monospace;
        }
        .info {
            background: #d1ecf1;
            border: 1px solid #bee5eb;
            color: #0c5460;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }
        .debug-log {
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            padding: 10px;
            border-radius: 4px;
            height: 200px;
            overflow-y: auto;
            font-family: monospace;
            font-size: 12px;
            margin-top: 10px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
            <p>Master blockchain for tenant networks - Debug Mode</p>
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
        <button class="debug-btn" onclick="debugBlocks()">Debug Blocks</button>
        <button class="debug-btn" onclick="testAllEndpoints()">Test Endpoints</button>
        
        <div class="stats" id="stats">
            <div class="stat-card">
                <div class="stat-value" id="block-height">-</div>
                <div class="stat-label">Tenant Blocks</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="validator-count">-</div>
                <div class="stat-label">Active Validators</div>
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
            <h3>Validator Status</h3>
            <div id="validator-status-list"></div>
        </div>
        
        <div class="section">
            <h2>Recent Tenant Blocks</h2>
            <div id="blocks-debug-info" class="info" style="display: none;"></div>
            <div id="recent-blocks">Loading...</div>
        </div>

        <div class="section">
            <h2>Cross-Network Trades</h2>
            <div id="cross-network-trades">Loading...</div>
        </div>
        
        <div class="section">
            <h2>Tenant Summaries</h2>
            <div id="tenant-summaries">Loading...</div>
        </div>
        
        <div class="section">
            <h3>Debug Log</h3>
            <div id="debug-log" class="debug-log"></div>
        </div>
    </div>

    <script>
        // Configuration
        let API_BASE = '';
        let debugLog = [];
        
        function log(message, level = 'INFO') {
            const timestamp = new Date().toLocaleTimeString();
            const entry = `[${timestamp}] ${level}: ${message}`;
            debugLog.push(entry);
            
            const logDiv = document.getElementById('debug-log');
            logDiv.innerHTML = debugLog.slice(-50).join('\n');
            logDiv.scrollTop = logDiv.scrollHeight;
            
            console.log(entry);
        }
        
        function initializeApiUrl() {
            API_BASE = `http://${window.location.hostname}:8080`;
            document.getElementById('api-url').value = API_BASE;
            log(`Initialized API URL: ${API_BASE}`);
        }
        
        function updateApiUrl() {
            const newUrl = document.getElementById('api-url').value.trim();
            if (newUrl) {
                API_BASE = newUrl;
                log(`Updated API URL: ${API_BASE}`);
                testConnection();
            }
        }
        
        async function testConnection() {
            log('Testing connection...');
            const statusEl = document.getElementById('connection-status');
            const textEl = document.getElementById('connection-text');
            const detailsEl = document.getElementById('connection-details');
            
            try {
                const response = await fetch(`${API_BASE}/health`, {
                    method: 'GET',
                    mode: 'cors'
                });
                
                if (response.ok) {
                    const data = await response.json();
                    statusEl.className = 'connection-status connected';
                    textEl.textContent = 'Connected';
                    detailsEl.innerHTML = `<div class="success">Health check passed. Status: ${data.status}</div>`;
                    log('Connection test successful', 'SUCCESS');
                } else {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
            } catch (error) {
                statusEl.className = 'connection-status';
                textEl.textContent = 'Failed';
                detailsEl.innerHTML = `<div class="error">Error: ${error.message}</div>`;
                log(`Connection test failed: ${error.message}`, 'ERROR');
            }
        }
        
        async function loadDashboard() {
            log('Loading dashboard data...');
            
            try {
                await Promise.all([
                    loadValidatorStatus(),
                    loadBlocks(),
                    loadTenants(),
                    loadCrossNetworkTrades()  // Add this line
                ]);
                log('Dashboard loaded successfully', 'SUCCESS');
            } catch (error) {
                log(`Error loading dashboard: ${error.message}`, 'ERROR');
                showError('Failed to load dashboard data: ' + error.message);
            }
        }
        
        async function loadValidatorStatus() {
            try {
                log('Fetching validator status...');
                const response = await fetch(`${API_BASE}/api/status`);
                if (response.ok) {
                    const status = await response.json();
                    
                    document.getElementById('block-height').textContent = status.height || 0;
                    document.getElementById('validator-count').textContent = status.active_validators || 1;
                    document.getElementById('pending-updates').textContent = status.total_transactions || 0;
                    
                    const validatorStatusHtml = `
                        <div class="validator-status">
                            Validator Active: ${status.validator}<br>
                            Tenant Blocks: ${status.height}<br>
                            Enterprise Blocks: ${status.total_blocks}<br>
                            Total Transactions: ${status.total_transactions}<br>
                            Active Validators: ${status.active_validators}<br>
                            Active Tenants: ${status.active_tenants}<br>
                            Chain Health: ${status.chain_health || 'healthy'}
                        </div>
                    `;
                    
                    document.getElementById('validator-status-list').innerHTML = validatorStatusHtml;
                    log(`Validator status loaded: height=${status.height}, validator=${status.validator}`, 'SUCCESS');
                } else {
                    throw new Error(`HTTP ${response.status}`);
                }
            } catch (error) {
                log(`Error loading validator status: ${error.message}`, 'ERROR');
                document.getElementById('validator-status-list').innerHTML = 
                    `<div class="error">Failed to load validator status: ${error.message}</div>`;
                throw error;
            }
        }
        
        async function loadBlocks() {
            try {
                log('Fetching tenant blocks...');
                const response = await fetch(`${API_BASE}/api/blocks?limit=10`);
                const blocks = await response.json();
                
                const container = document.getElementById('recent-blocks');
                container.innerHTML = '';
                
                log(`Received response with ${blocks.length} blocks`);
                
                if (!blocks || blocks.length === 0) {
                    container.innerHTML = '<div class="warning">No tenant blocks found - Check data flow</div>';
                    log('No tenant blocks received', 'WARNING');
                    showBlocksDebugInfo();
                    return;
                }
                
                blocks.reverse().forEach(block => {
                    const blockDiv = document.createElement('div');
                    blockDiv.className = 'block';
                    
                    const blockHash = block.block_hash || block.hash || 'N/A';
                    const previousHash = block.previous_hash || 'N/A';
                    const transactions = block.transactions || [];
                    
                    blockDiv.innerHTML = `
                        <div style="font-weight: bold; margin-bottom: 5px;">
                            Tenant Block #${block.block_id || block.id || '0'} - Network: ${block.network_id || 'Unknown'}
                        </div>
                        <div>Hash: ${blockHash.length > 16 ? blockHash.substring(0, 16) + '...' : blockHash}</div>
                        <div>Previous: ${previousHash.length > 16 ? previousHash.substring(0, 16) + '...' : previousHash}</div>
                        <div>Timestamp: ${new Date((block.timestamp || 0) * 1000).toLocaleString()}</div>
                        <div>Transactions: ${transactions.length}</div>
                        <div>From Peer: ${block.from_peer || 'Unknown'}</div>
                        ${transactions.length > 0 ? `
                            <div style="margin-top: 8px;">
                                <strong>Transaction Data:</strong><br>
                                ${transactions.map(tx => `<span style="font-size: 12px; color: #666;">• ${tx}</span>`).join('<br>')}
                            </div>
                        ` : ''}
                    `;
                    container.appendChild(blockDiv);
                });
                
                log(`Loaded ${blocks.length} tenant blocks`, 'SUCCESS');
            } catch (error) {
                log(`Error loading blocks: ${error.message}`, 'ERROR');
                document.getElementById('recent-blocks').innerHTML = 
                    `<div class="error">Failed to load blocks: ${error.message}</div>`;
            }
        }
        
        async function loadTenants() {
            try {
                log('Fetching tenant summaries...');
                const response = await fetch(`${API_BASE}/api/tenants`);
                
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
                
                const data = await response.json();
                const container = document.getElementById('tenant-summaries');
                container.innerHTML = '';

                document.getElementById('tenant-count').textContent = data.total_tenants || 0;

                if (!data.tenants || data.tenants.length === 0) {
                    container.innerHTML = '<div class="info">No active tenants - connect browser clients and create transactions</div>';
                    log('No tenant summaries found', 'WARNING');
                    return;
                }

                data.tenants.forEach(tenant => {
                    const tenantDiv = document.createElement('div');
                    tenantDiv.className = 'tenant-summary';

                    let recentMessagesHtml = '';
                    if (tenant.recent_messages && tenant.recent_messages.length > 0) {
                        recentMessagesHtml = `
                            <div style="margin-top: 8px;">
                                <strong>Recent Activity:</strong><br>
                                ${tenant.recent_messages.map(msg => 
                                    `<span style="font-size: 12px; color: #666;">• ${msg}</span>`
                                ).join('<br>')}
                            </div>
                        `;
                    }

                    tenantDiv.innerHTML = `
                        <div><strong>${tenant.tenant_id}</strong></div>
                        <div>Blocks: ${tenant.block_count} | Transactions: ${tenant.transaction_count}</div>
                        <div>Last Activity: ${new Date(tenant.last_activity * 1000).toLocaleString()}</div>
                        ${recentMessagesHtml}
                    `;
                    container.appendChild(tenantDiv);
                });

                log(`Loaded ${data.tenants.length} tenant summaries`, 'SUCCESS');

            } catch (error) {
                log(`Error loading tenants: ${error.message}`, 'ERROR');
                document.getElementById('tenant-summaries').innerHTML = 
                    `<div class="error">Failed to load tenant data: ${error.message}</div>`;
                
                document.getElementById('tenant-count').textContent = 0;
            }
        }

        // Add this JavaScript function to the dashboard (in src/enterprise_bc/dashboard.rs)
// Find the loadTenants function and add this after it:

async function loadCrossNetworkTrades() {
    try {
        log('Fetching cross-network trades...');
        const response = await fetch(`${API_BASE}/api/cross-network-trades`, {
            method: 'GET',
            mode: 'cors'
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        log(`Cross-network trades response: ${JSON.stringify(data)}`);

        const container = document.getElementById('cross-network-trades');
        container.innerHTML = '';

        if (!data.cross_network_trades || data.cross_network_trades.length === 0) {
            container.innerHTML = '<div class="info">No cross-network trades executed yet - trades will appear here automatically when orders match across networks</div>';
            log('No cross-network trades found');
            return;
        }

        data.cross_network_trades.forEach((trade, index) => {
            const tradeDiv = document.createElement('div');
            tradeDiv.className = 'block';
            tradeDiv.style.background = '#e8f5e8';
            tradeDiv.style.borderLeft = '4px solid #28a745';
            tradeDiv.style.marginBottom = '10px';

            const timestamp = trade.timestamp ? new Date(trade.timestamp * 1000).toLocaleString() : 'Unknown';

            tradeDiv.innerHTML = `
                <div style="font-weight: bold; margin-bottom: 8px; color: #28a745; font-size: 16px;">
                    🔄 Cross-Network Trade #${index + 1}
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; font-size: 14px;">
                    <div><strong>Asset:</strong> ${trade.asset}</div>
                    <div><strong>Quantity:</strong> ${trade.quantity}</div>
                    <div><strong>Price:</strong> ${trade.price}</div>
                    <div><strong>Enterprise Block:</strong> #${trade.enterprise_block}</div>
                </div>
                <div style="margin-top: 8px; font-size: 14px;">
                    <div><strong>Buyer Network:</strong> <span style="color: #007bff;">${trade.buyer_network}</span></div>
                    <div><strong>Seller Network:</strong> <span style="color: #dc3545;">${trade.seller_network}</span></div>
                </div>
                <div style="margin-top: 8px; font-size: 12px; color: #666;">
                    <div><strong>Trade ID:</strong> ${trade.trade_id}</div>
                    <div><strong>Executed:</strong> ${timestamp}</div>
                    <div><strong>Details:</strong> ${trade.transaction_data}</div>
                </div>
            `;
            container.appendChild(tradeDiv);
        });

        log(`Loaded ${data.cross_network_trades.length} cross-network trades`, 'SUCCESS');

    } catch (error) {
        log(`Error loading cross-network trades: ${error.message}`, 'ERROR');
        document.getElementById('cross-network-trades').innerHTML =
            `<div class="error">Failed to load cross-network trades: ${error.message}<br>Check if enterprise validator is running and API is accessible.</div>`;
    }
}

// Also update the loadDashboard function to include cross-network trades:
// Find this function and make sure it looks like this:

async function loadDashboard() {
    log('Loading dashboard data...');

    try {
        await Promise.all([
            loadValidatorStatus(),
            loadBlocks(),
            loadTenants(),
            loadCrossNetworkTrades()  // Make sure this line is added
        ]);
        log('Dashboard loaded successfully', 'SUCCESS');
    } catch (error) {
        log(`Error loading dashboard: ${error.message}`, 'ERROR');
        showError('Failed to load dashboard data: ' + error.message);
    }
}

        async function debugBlocks() {
            log('Running blocks debug check...');
            
            try {
                // Test blocks endpoint specifically
                const response = await fetch(`${API_BASE}/api/blocks?limit=5`);
                const data = await response.json();
                
                log(`Blocks endpoint status: ${response.status}`);
                log(`Blocks data type: ${Array.isArray(data) ? 'array' : typeof data}`);
                log(`Blocks count: ${data.length || 0}`);
                
                if (data.length === 0) {
                    log('No blocks found. Checking possible causes...', 'WARNING');
                    showBlocksDebugInfo();
                } else {
                    log(`First block: ${JSON.stringify(data[0], null, 2)}`);
                }
                
                await loadBlocks();
            } catch (error) {
                log(`Blocks debug error: ${error.message}`, 'ERROR');
            }
        }
        
        function showBlocksDebugInfo() {
            const debugInfo = document.getElementById('blocks-debug-info');
            debugInfo.style.display = 'block';
            debugInfo.innerHTML = `
                <strong>No blocks found. Debug checklist:</strong><br>
                1. Check if tracker is running and configured with ENTERPRISE_BC_URL<br>
                2. Verify browser clients are creating transactions and mining blocks<br>
                3. Check network connectivity between tracker and enterprise BC<br>
                4. Ensure enterprise validator is receiving updates at /api/tenant-blockchain-update
            `;
        }
        
        async function testAllEndpoints() {
            log('Testing all API endpoints...');
            
            const endpoints = [
                { path: '/health', name: 'Health Check' },
                { path: '/api/status', name: 'Status' },
                { path: '/api/blocks?limit=1', name: 'Blocks' },
                { path: '/api/tenants', name: 'Tenants' }
            ];
            
            for (const endpoint of endpoints) {
                try {
                    const response = await fetch(`${API_BASE}${endpoint.path}`);
                    const status = response.ok ? 'OK' : 'FAILED';
                    log(`${endpoint.name}: HTTP ${response.status} ${status}`, response.ok ? 'SUCCESS' : 'ERROR');
                } catch (error) {
                    log(`${endpoint.name}: ERROR - ${error.message}`, 'ERROR');
                }
            }
        }

        function showError(message) {
            document.getElementById('recent-blocks').innerHTML = `<div class="error">${message}</div>`;
            document.getElementById('tenant-summaries').innerHTML = `<div class="error">${message}</div>`;
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

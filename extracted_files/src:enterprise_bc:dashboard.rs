use warp::Filter;
use tracing::info;

pub async fn start_dashboard(port: u16) {
    info!("Starting dashboard on port {}", port);
    
    let dashboard_html = warp::path::end()
        .map(|| warp::reply::html(DASHBOARD_HTML));
    
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));
    
    let routes = dashboard_html.or(static_files);
    
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
        .refresh-btn {
            background: #3498db;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin-bottom: 20px;
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
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
            <p>Master blockchain for tenant networks - Validator Status</p>
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
                <div class="stat-label">Block Height</div>
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
                <div class="stat-label">Pending Transactions</div>
            </div>
        </div>
        
        <div class="section">
            <h3>Validator Status</h3>
            <div id="validator-status-list"></div>
        </div>
        
        <div class="section">
            <h2>Recent Blocks</h2>
            <div id="recent-blocks">Loading...</div>
        </div>
        
        <div class="section">
            <h2>Tenant Summaries</h2>
            <div id="tenant-summaries">Loading...</div>
        </div>
    </div>

    <script>
        // Configuration
        let API_BASE = '';
        let errorCount = 0;
        
        // Initialize API URL - works for both manual and Docker deployment
        function initializeApiUrl() {
            // Default to the host and port 8080 for manual validator
            // For Docker deployment, this would be changed to the load balancer port
            API_BASE = `http://${window.location.hostname}:8080`;
            document.getElementById('api-url').value = API_BASE;
            
            console.log(`Initialized API URL: ${API_BASE}`);
        }
        
        function updateApiUrl() {
            const newUrl = document.getElementById('api-url').value.trim();
            if (newUrl) {
                API_BASE = newUrl;
                console.log(`Updated API URL: ${API_BASE}`);
                testConnection();
            }
        }
        
        async function testConnection() {
            console.log('Testing connection...');
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
                    textEl.textContent = 'Connected ✅';
                    detailsEl.innerHTML = `<div class="success">Health check passed. Status: ${data.status}</div>`;
                    console.log('Connection test successful');
                } else {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
            } catch (error) {
                statusEl.className = 'connection-status';
                textEl.textContent = 'Failed ❌';
                detailsEl.innerHTML = `<div class="error">Error: ${error.message}</div>`;
                console.log(`Connection test failed: ${error.message}`);
                errorCount++;
            }
        }
        
        async function loadDashboard() {
            console.log('Loading dashboard data...');
            
            try {
                await Promise.all([
                    loadValidatorStatus(),
                    loadBlocks(),
                    loadTenants()
                ]);
                console.log('Dashboard loaded successfully');
            } catch (error) {
                console.log(`Error loading dashboard: ${error.message}`);
                showError('Failed to load dashboard data: ' + error.message);
                errorCount++;
            }
        }
        
        async function loadValidatorStatus() {
            try {
                const response = await fetch(`${API_BASE}/api/status`);
                if (response.ok) {
                    const status = await response.json();
                    
                    // Update UI with validator data
                    document.getElementById('block-height').textContent = status.height || 0;
                    document.getElementById('validator-count').textContent = status.active_validators || 1;
                    document.getElementById('pending-updates').textContent = status.pending_transactions || 0;
                    
                    // Display comprehensive validator status
                    const validatorStatusHtml = `
                        <div class="validator-status">
                            ✅ Validator Active: ${status.validator}<br>
                            📊 Height: ${status.height}<br>
                            📦 Total Blocks: ${status.total_blocks}<br>
                            🔄 Total Transactions: ${status.total_transactions}<br>
                            👥 Active Validators: ${status.active_validators}<br>
                            🏢 Active Tenants: ${status.active_tenants}<br>
                            ⏰ Smart Contracts Ready: ${status.ready_for_smart_contracts ? 'Yes' : 'No'}<br>
                            💚 Chain Health: ${status.chain_health || 'healthy'}
                        </div>
                    `;
                    
                    document.getElementById('validator-status-list').innerHTML = validatorStatusHtml;
                    
                    console.log(`✅ Validator status loaded: height=${status.height}, validator=${status.validator}`);
                } else {
                    throw new Error(`HTTP ${response.status}`);
                }
            } catch (error) {
                console.log(`Error loading validator status: ${error.message}`);
                document.getElementById('validator-status-list').innerHTML = 
                    `<div class="error">❌ Failed to load validator status: ${error.message}</div>`;
                throw error;
            }
        }
        
        async function loadBlocks() {
            try {
                const response = await fetch(`${API_BASE}/api/blocks?limit=5`);
                const blocks = await response.json();
                
                const container = document.getElementById('recent-blocks');
                container.innerHTML = '';
                
                if (!blocks || blocks.length === 0) {
                    container.innerHTML = '<div class="info">No blocks yet - waiting for tenant transactions</div>';
                    return;
                }
                
                blocks.reverse().forEach(block => {
                    const blockDiv = document.createElement('div');
                    blockDiv.className = 'block';
                    
                    // Enhanced block display with transaction content
                    let transactionHtml = '';
                    if (block.transactions && block.transactions.length > 0) {
                        transactionHtml = `
                            <div style="margin-top: 8px;">
                                <strong>Transactions (${block.transactions.length}):</strong><br>
                                ${block.transactions.map(tx => 
                                    `<span style="font-size: 12px; color: #666;">• ${tx.tenant_network}: "${tx.transaction_data}" (from ${tx.from_peer})</span>`
                                ).join('<br>')}
                            </div>
                        `;
                    }
                    
                    blockDiv.innerHTML = `
                        <div style="font-weight: bold; margin-bottom: 5px;">
                            Block #${block.height} - Validated by ${block.validator}
                        </div>
                        <div>Hash: ${block.hash.substring(0, 16)}...</div>
                        <div>Previous: ${block.previous_hash.substring(0, 16)}...</div>
                        <div>Timestamp: ${new Date(block.timestamp * 1000).toLocaleString()}</div>
                        <div>Transaction Count: ${block.transaction_count}</div>
                        <div>Merkle Root: ${block.merkle_root.substring(0, 16)}...</div>
                        <div>Nonce: ${block.nonce}</div>
                        ${transactionHtml}
                    `;
                    container.appendChild(blockDiv);
                });
                
                console.log(`Loaded ${blocks.length} blocks`);
            } catch (error) {
                console.error('Error loading blocks:', error);
                document.getElementById('recent-blocks').innerHTML = 
                    `<div class="error">Failed to load blocks: ${error.message}</div>`;
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

                // Update tenant count in stats
                document.getElementById('tenant-count').textContent = data.total_tenants || 0;

                if (!data.tenants || data.tenants.length === 0) {
                    container.innerHTML = '<div class="info">No active tenants - connect browser clients and create transactions</div>';
                    return;
                }

                data.tenants.forEach(tenant => {
                    const tenantDiv = document.createElement('div');
                    tenantDiv.className = 'tenant-summary';

                    // Enhanced display with actual block content
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
                        <div><strong>🏢 ${tenant.tenant_id}</strong></div>
                        <div>📊 Blocks: ${tenant.block_count} | Transactions: ${tenant.transaction_count}</div>
                        <div>⏰ Last Activity: ${new Date(tenant.last_activity * 1000).toLocaleString()}</div>
                        ${recentMessagesHtml}
                    `;
                    container.appendChild(tenantDiv);
                });

                console.log(`Loaded ${data.tenants.length} tenant summaries`);

            } catch (error) {
                console.error('Error loading tenants:', error);
                document.getElementById('tenant-summaries').innerHTML = 
                    `<div class="error">Failed to load tenant data: ${error.message}</div>`;
                
                // Set tenant count to 0 on error
                document.getElementById('tenant-count').textContent = 0;
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

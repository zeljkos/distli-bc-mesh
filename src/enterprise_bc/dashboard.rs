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
        .debug-section {
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            padding: 15px;
            border-radius: 8px;
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
        .log {
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            padding: 10px;
            border-radius: 4px;
            height: 200px;
            overflow-y: auto;
            font-family: monospace;
            font-size: 12px;
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
        .block-content {
            background: #f8f9fa;
            padding: 8px;
            border-radius: 4px;
            margin: 5px 0;
            font-family: monospace;
            font-size: 12px;
            border: 1px solid #dee2e6;
        }

        .activity-item {
            font-size: 12px;
            color: #666;
            margin: 2px 0;
            padding: 2px 0;
        }

    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
            <p>Master blockchain for tenant networks</p>
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
                <div class="stat-label">Pending Updates</div>
            </div>
        </div>
        
        <div class="debug-section">
            <h3>Debug Information</h3>
            <div>Current API URL: <strong id="current-api-url">Not set</strong></div>
            <div>Last Update: <strong id="last-update">Never</strong></div>
            <div>Error Count: <strong id="error-count">0</strong></div>
            <div class="log" id="debug-log"></div>
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
        
        // Initialize API URL
        function initializeApiUrl() {
            // Try different possible URLs
            const possibleUrls = [
                `http://${window.location.hostname}:8080`,
                'http://localhost:8080',
                'http://192.168.200.133:8080'
            ];
            
            API_BASE = possibleUrls[0];
            document.getElementById('api-url').value = API_BASE;
            document.getElementById('current-api-url').textContent = API_BASE;
            
            debugLog(`Initialized API URL: ${API_BASE}`);
        }
        
        function updateApiUrl() {
            const newUrl = document.getElementById('api-url').value.trim();
            if (newUrl) {
                API_BASE = newUrl;
                document.getElementById('current-api-url').textContent = API_BASE;
                debugLog(`Updated API URL: ${API_BASE}`);
                testConnection();
            }
        }
        
        async function testConnection() {
            debugLog('Testing connection...');
            const statusEl = document.getElementById('connection-status');
            const textEl = document.getElementById('connection-text');
            const detailsEl = document.getElementById('connection-details');
            
            try {
                const response = await fetch(`${API_BASE}/health`, {
                    method: 'GET',
                    timeout: 5000
                });
                
                if (response.ok) {
                    const data = await response.json();
                    statusEl.className = 'connection-status connected';
                    textEl.textContent = 'Connected ✅';
                    detailsEl.innerHTML = `<div class="success">Health check passed. Status: ${data.status}</div>`;
                    debugLog('Connection test successful');
                } else {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }
            } catch (error) {
                statusEl.className = 'connection-status';
                textEl.textContent = 'Failed ❌';
                detailsEl.innerHTML = `<div class="error">Error: ${error.message}</div>`;
                debugLog(`Connection test failed: ${error.message}`);
                errorCount++;
                document.getElementById('error-count').textContent = errorCount;
            }
        }
        
        async function loadDashboard() {
            debugLog('Loading dashboard data...');
            document.getElementById('last-update').textContent = new Date().toLocaleTimeString();
            
            try {
                await Promise.all([
                    loadStatus(),
                    loadBlocks(),
                    loadTenants()
                ]);
                debugLog('Dashboard loaded successfully');
            } catch (error) {
                debugLog(`Error loading dashboard: ${error.message}`);
                showError('Failed to load dashboard data: ' + error.message);
                errorCount++;
                document.getElementById('error-count').textContent = errorCount;
            }
        }
        
        async function loadStatus() {
            try {
                debugLog('Fetching status...');
                const response = await fetch(`${API_BASE}/api/status`);
                
                if (!response.ok) {
                    throw new Error(`Status API returned ${response.status}: ${response.statusText}`);
                }
                
                const status = await response.json();
                debugLog(`Status loaded: height=${status.height}, tenants=${status.active_tenants}`);
                
                document.getElementById('block-height').textContent = status.height || 0;
                document.getElementById('tenant-count').textContent = status.active_tenants || 0;
                document.getElementById('pending-updates').textContent = status.pending_updates || 0;
                 // ADD THIS LINE to use actual validator count from API:
                document.getElementById('validator-count').textContent = status.active_validators || 1;
            } catch (error) {
                debugLog(`Error loading status: ${error.message}`);
                throw error;
            }
        }
       // 
       //
        // ENHANCED: Better Recent Blocks display
async function loadBlocks() {
    try {
        const response = await fetch(`${API_BASE}/api/blocks?limit=5`);
        const blocks = await response.json();
        
        const container = document.getElementById('recent-blocks');
        container.innerHTML = '';
        
        if (blocks.length === 0) {
            container.innerHTML = '<p>No blocks yet</p>';
            return;
        }
        
        blocks.reverse().forEach(block => {
            const blockDiv = document.createElement('div');
            blockDiv.className = 'block';
            
            // Enhanced block display with tenant content
            let tenantContentHtml = '';
            if (block.tenant_summaries && block.tenant_summaries.length > 0) {
                tenantContentHtml = block.tenant_summaries.map(summary => `
                    <div class="block-content">
                        <strong>${summary.tenant_id}:</strong><br>
                        ${summary.messages.map(msg => `<span class="activity-item">• ${msg}</span>`).join('<br>')}
                    </div>
                `).join('');
            }
            
            blockDiv.innerHTML = `
                <div class="block-header">
                    Block #${block.height} - Validated by ${block.validator}
                </div>
                <div>Hash: ${block.hash.substring(0, 16)}...</div>
                <div>Timestamp: ${new Date(block.timestamp * 1000).toLocaleString()}</div>
                <div>Tenant Summaries: ${block.tenant_summaries.length}</div>
                ${tenantContentHtml}
            `;
            container.appendChild(blockDiv);
        });
    } catch (error) {
        console.error('Error loading blocks:', error);
    }
}
        ///
        async function loadTenants() {
          try {
            const response = await fetch(`${API_BASE}/api/tenants`);
            const data = await response.json();

            const container = document.getElementById('tenant-summaries');
            container.innerHTML = '';

        if (!data.tenants || data.tenants.length === 0) {
            container.innerHTML = '<p>No active tenants</p>';
            return;
        }

        data.tenants.forEach(tenant => {
            const tenantDiv = document.createElement('div');
            tenantDiv.className = 'tenant-summary';

            // Enhanced display with actual block content
            let recentMessagesHtml = '';
            if (tenant.recent_messages && tenant.recent_messages.length > 0) {
                const messages = tenant.recent_messages.slice(-3);
                recentMessagesHtml = `
                    <div style="margin-top: 8px;">
                        <strong>Recent Activity:</strong><br>
                        ${messages.map(msg => `<span style="font-size: 12px; color: #666;">• ${msg}</span>`).join('<br>')}
                    </div>
                `;
            }

            tenantDiv.innerHTML = `
                <div><strong>${tenant.tenant_id}</strong></div>
                <div>Blocks: ${tenant.block_count} | Transactions: ${tenant.transaction_count} | Peers: ${tenant.peer_count}</div>
                <div>Last Activity: ${new Date(tenant.last_activity * 1000).toLocaleString()}</div>
                ${recentMessagesHtml}
            `;
            container.appendChild(tenantDiv);
        });

    } catch (error) {
        console.error('Error loading tenants:', error);
    }
}

        
        function showError(message) {
            document.getElementById('recent-blocks').innerHTML = `<div class="error">${message}</div>`;
            document.getElementById('tenant-summaries').innerHTML = `<div class="error">${message}</div>`;
        }
        
        function debugLog(message) {
            const logDiv = document.getElementById('debug-log');
            const time = new Date().toLocaleTimeString();
            logDiv.innerHTML += `${time}: ${message}\n`;
            logDiv.scrollTop = logDiv.scrollHeight;
            console.log(`[Dashboard] ${message}`);
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

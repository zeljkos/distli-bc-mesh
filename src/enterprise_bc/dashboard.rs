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
        .block {
            border: 1px solid #ddd;
            padding: 15px;
            margin: 10px 0;
            border-radius: 4px;
            background: #f9f9f9;
        }
        .block-header {
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 10px;
        }
        .tenant-summary {
            background: #e8f4fd;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
            border-left: 4px solid #3498db;
        }
        .error {
            color: #e74c3c;
            text-align: center;
            padding: 20px;
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
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Enterprise Blockchain Dashboard</h1>
            <p>Master blockchain for tenant networks</p>
        </div>
        
        <button class="refresh-btn" onclick="loadDashboard()">Refresh Data</button>
        
        <div class="stats" id="stats">
            <div class="stat-card">
                <div class="stat-value" id="block-height">-</div>
                <div class="stat-label">Block Height</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="validator-count">3</div>
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
        // Use current host for API calls instead of hardcoded localhost
        const API_BASE = `http://${window.location.hostname}:8080`;
        
        async function loadDashboard() {
            try {
                await Promise.all([
                    loadStatus(),
                    loadBlocks(),
                    loadTenants()
                ]);
            } catch (error) {
                console.error('Error loading dashboard:', error);
                showError('Failed to load dashboard data');
            }
        }
        
        async function loadStatus() {
            try {
                const response = await fetch(`${API_BASE}/api/status`);
                const status = await response.json();
                
                document.getElementById('block-height').textContent = status.height || 0;
                document.getElementById('tenant-count').textContent = status.active_tenants || 0;
                document.getElementById('pending-updates').textContent = status.pending_updates || 0;
            } catch (error) {
                console.error('Error loading status:', error);
            }
        }
        
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
                    blockDiv.innerHTML = `
                        <div class="block-header">
                            Block #${block.height} - Validated by ${block.validator}
                        </div>
                        <div>Hash: ${block.hash.substring(0, 16)}...</div>
                        <div>Timestamp: ${new Date(block.timestamp * 1000).toLocaleString()}</div>
                        <div>Tenant Summaries: ${block.tenant_summaries.length}</div>
                    `;
                    container.appendChild(blockDiv);
                });
            } catch (error) {
                console.error('Error loading blocks:', error);
            }
        }
        
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
                    tenantDiv.innerHTML = `
                        <div><strong>${tenant.tenant_id}</strong></div>
                        <div>Blocks: ${tenant.block_count} | Transactions: ${tenant.transaction_count} | Peers: ${tenant.peer_count}</div>
                        <div>Last Activity: ${new Date(tenant.last_activity * 1000).toLocaleString()}</div>
                        <div>Recent Messages: ${tenant.recent_messages.slice(-3).join(', ') || 'None'}</div>
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
        
        // Auto-refresh every 30 seconds
        setInterval(loadDashboard, 30000);
        
        // Load initial data
        loadDashboard();
    </script>
</body>
</html>
"#;

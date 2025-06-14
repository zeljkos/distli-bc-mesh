# distli-mesh-bc

A simple multi-tenant distributed blockchain mesh network using WebRTC for peer-to-peer communication.

## Features

- **Multi-tenant networks**: Create isolated blockchain networks
- **Real-time network discovery**: Dropdown automatically updates when networks are created/removed
- **Network discovery**: Dropdown list of active networks with peer counts
- **Peer discovery**: Network-scoped peer discovery via WebSocket tracker  
- **Direct P2P communication**: WebRTC data channels between browser peers
- **Network isolation**: Peers only connect within the same network
- **Simple proof-of-work blockchain**: Basic mining and transaction system
- **Cross-machine support**: Connect peers from different machines

## Multi-Tenant Architecture

Each network operates as an isolated blockchain mesh:
- **Network isolation**: Peers can only see and connect to peers in the same network
- **Real-time discovery**: Network dropdown updates automatically across all browsers
- **Separate blockchains**: Each network maintains its own blockchain state
- **Independent mining**: Mining and transactions are scoped to each network
- **Network management**: Tracker manages multiple networks simultaneously
- **Live peer counts**: See real-time peer counts for each network

## Setup

1. Create the project:
```bash
cargo new distli-mesh-bc
cd distli-mesh-bc
```

2. Replace `Cargo.toml` with the provided configuration

3. Create the source files in `src/`:
   - `main.rs`
   - `types.rs` 
   - `blockchain.rs`
   - `tracker.rs`

4. Create the `public/` directory and add `index.html`

5. Run the project:
```bash
cargo run
```

6. Open browser tabs to `http://SERVER_IP:3030`

## Usage

### Single Machine
1. Open multiple browser tabs to `http://localhost:3030`
2. Enter different network names (e.g., "network-a", "network-b")
3. Peers with the same network name will form isolated meshes

### Multiple Machines
1. Find server IP: `ip addr show` (Linux) or `ipconfig` (Windows)
2. Start server: `cargo run`
3. Open firewall port 3030 if needed
4. On other machines: browse to `http://SERVER_IP:3030`
5. Enter server IP in "Server" field
6. Choose network names to create isolated groups

### Network Operations
1. **Connect**: Connect to tracker server
2. **Select Network**: Choose from dropdown of active networks OR enter new network name
3. **Refresh Networks**: Click ↻ to update the list of available networks
4. **Join Network**: Join the selected/entered network
5. **Discover Peers**: Find other peers in your network
6. **Connect All**: Establish P2P connections within network
7. **Send transactions**: Create transactions within your network
8. **Mine blocks**: Mine blocks for your network's blockchain

## Example Networks

```
Network "company-a":     Network "company-b":
├── Peer 1               ├── Peer 5  
├── Peer 2               ├── Peer 6
└── Peer 3               └── Peer 7

Network "public":
├── Peer 8
├── Peer 9  
├── Peer 10
└── Peer 11
```

Each network maintains completely separate:
- Peer discovery and connections
- Blockchain state and transactions  
- Mining operations and blocks

## API Endpoints

- `GET /api/networks` - View all active networks and peer counts (detailed info)
- `GET /api/network-list` - Get list of networks for dropdown (simplified format)
- `WS /ws` - WebSocket endpoint for peer connections

## Project Structure

```
distli-mesh-bc/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs           # Entry point
│   ├── types.rs          # Data structures with network support
│   ├── blockchain.rs     # Simple blockchain implementation  
│   └── tracker.rs        # Multi-tenant WebSocket tracker
└── public/
    └── index.html        # Multi-tenant web interface
```

# 📱 GSM Roaming Smart Contract System

## Overview

The GSM Roaming Smart Contract System provides blockchain-based billing and session management for mobile network roaming scenarios. When a mobile device connects to a foreign network, smart contracts automatically handle authentication, rate setting, minute-by-minute billing, and settlement between network operators.

## 🚀 Features

- **Smart Contract-Based Billing**: Automated billing using blockchain smart contracts
- **Multi-Network Support**: Configure rates between different network operators
- **Real-Time Session Management**: Connect, disconnect, and monitor active roaming sessions
- **Automated Minute Billing**: Every minute triggers a blockchain transaction for billing
- **Transparent Pricing**: All rates and billing history stored on blockchain
- **Wallet Integration**: Direct payments between guest and host network wallets

## 🏗️ Architecture

### Core Components

1. **GSM Roaming Smart Contract** (`gsm_roaming`)
   - Session management
   - Rate configuration
   - Billing logic
   - Event logging

2. **Transaction Types**
   - `RoamingConnect`: Device connects to foreign network
   - `RoamingDisconnect`: Device disconnects, final billing
   - `RoamingBilling`: Per-minute billing transactions

3. **Web Interface**
   - Network configuration panel
   - Device connection management
   - Real-time session monitoring
   - Billing history display

## 📡 Usage Scenarios

### Scenario 1: Device Roaming Connection

1. **Mobile device** (IMSI: 310410123456789) from **Vodafone** travels abroad
2. Device attempts to connect to **T-Mobile** network via antenna **ANT-001**
3. Smart contract:
   - Validates connection request
   - Looks up rate: Vodafone → T-Mobile (e.g., 15 units/minute)
   - Creates roaming session
   - Records guest wallet (Vodafone) and host wallet (T-Mobile)

### Scenario 2: Automated Billing

1. Every minute, billing system calls `processMinuteBilling`
2. Smart contract:
   - Increments minute counter
   - Charges guest wallet 15 units
   - Credits host wallet 15 units
   - Creates blockchain transaction
   - Emits billing event

### Scenario 3: Session Termination

1. Device disconnects or moves out of coverage
2. Smart contract:
   - Calculates total session duration
   - Processes final billing
   - Archives session to billing history
   - Clears active session

## 🛠️ Getting Started

### 1. Deploy the System

```bash
# Build the project
cargo build --features native

# Run the blockchain network
cargo run --bin tracker --features native

# Start validator
TRACKER_URL="http://localhost:3030" cargo run --bin enterprise-validator --features native -- --id validator1 --port 8080 --stake 1000

# Start dashboard
cargo run --bin enterprise-dashboard --features native -- --port 9090
```

### 2. Test the Smart Contract

```bash
# Run the GSM roaming test example
cargo run --example gsm_roaming_test --features native
```

### 3. Use the Web Interface

1. Open `http://localhost:3030` in your browser
2. Navigate to the **GSM Roaming** tab
3. Configure network rates
4. Deploy roaming contract
5. Simulate device connections

## 📋 Smart Contract Functions

### Core Functions

#### `connect`
Establishes a new roaming session when a device connects to a foreign network.

**Parameters:**
- `imsi`: International Mobile Subscriber Identity
- `homeNetwork`: Device's home network operator
- `visitingNetwork`: Foreign network operator  
- `antennaId`: Physical antenna identifier
- `guestWallet`: Home network's wallet address
- `hostWallet`: Visiting network's wallet address

**Returns:**
- `sessionId`: Unique session identifier
- `ratePerMinute`: Billing rate for this network pair

#### `processMinuteBilling`
Processes billing for one minute of roaming usage.

**Parameters:**
- `sessionId`: Active session identifier

**Returns:**
- `minuteNumber`: Current minute number
- `amount`: Amount charged for this minute
- `totalCost`: Total session cost so far

#### `disconnect`
Terminates a roaming session and finalizes billing.

**Parameters:**
- `sessionId`: Session to terminate

**Returns:**
- `durationMinutes`: Total session duration
- `totalCost`: Final billing amount
- `billingRecord`: Complete billing record

#### `setRate`
Configures billing rate between network operators.

**Parameters:**
- `homeNetwork`: Home network name
- `visitingNetwork`: Visiting network name
- `ratePerMinute`: Rate in units per minute

### Query Functions

#### `getSession`
Retrieves information about an active session.

#### `getBillingHistory`
Gets historical billing records, optionally filtered by IMSI.

#### `getActiveSessions`
Lists all currently active roaming sessions.

## 💰 Billing Flow

### Minute-by-Minute Billing Process

```
1. Device connects → Smart contract creates session
2. Every 60 seconds:
   a. Billing system calls processMinuteBilling()
   b. Smart contract increments minute counter
   c. Creates blockchain transaction: guestWallet → hostWallet
   d. Emits MinuteBilled event
   e. Updates session total cost
3. Device disconnects → Smart contract finalizes billing
```

### Example Billing Scenario

**Network Rate:** Vodafone → T-Mobile = 15 units/minute
**Session Duration:** 8 minutes

| Minute | Action | Amount | Total Cost | Transaction |
|--------|---------|--------|------------|-------------|
| 1 | Connect + Bill | 15 | 15 | wallet_guest → wallet_host (15) |
| 2 | Bill | 15 | 30 | wallet_guest → wallet_host (15) |
| 3 | Bill | 15 | 45 | wallet_guest → wallet_host (15) |
| 4 | Bill | 15 | 60 | wallet_guest → wallet_host (15) |
| 5 | Bill | 15 | 75 | wallet_guest → wallet_host (15) |
| 6 | Bill | 15 | 90 | wallet_guest → wallet_host (15) |
| 7 | Bill | 15 | 105 | wallet_guest → wallet_host (15) |
| 8 | Bill + Disconnect | 15 | 120 | Final settlement |

## 🔧 Configuration

### Pre-configured Network Rates

The system comes with example rates:
- Vodafone ↔ T-Mobile: 15/12 units/minute
- Orange ↔ Verizon: 20/18 units/minute

### Adding New Network Operators

```javascript
// Via Web Interface
setRate("NewOperator", "ExistingOperator", 25);

// Via Smart Contract
{
    "function": "setRate",
    "params": {
        "homeNetwork": "NewOperator",
        "visitingNetwork": "ExistingOperator", 
        "ratePerMinute": 25
    }
}
```

## 🌐 Integration Points

### Enterprise Blockchain Integration

The roaming system integrates with the existing enterprise blockchain:
- Sessions are recorded as blockchain transactions
- Billing events trigger cross-chain notifications
- Multi-tenant support allows different operators on same blockchain

### Real-World Integration

**Network Equipment Integration:**
- Antenna systems call `connect` when device attaches
- Base stations trigger `disconnect` when device detaches
- Billing systems call `processMinuteBilling` on schedule

**Wallet Integration:**
- Guest wallets represent home network accounts
- Host wallets represent visiting network accounts
- Transactions create automatic settlements

## 📊 Monitoring & Analytics

### Available Metrics

- **Active Sessions**: Real-time count of roaming devices
- **Billing Volume**: Total units processed per time period
- **Network Utilization**: Usage by network operator pair
- **Revenue Analytics**: Income by operator and time period

### Event Types

- `RoamingConnected`: Device connects to foreign network
- `MinuteBilled`: Per-minute billing completed
- `RoamingDisconnected`: Device disconnects
- `RateSet`: Billing rate configured

## 🚨 Error Handling

### Common Error Scenarios

1. **Session Not Found**: Attempting to bill/disconnect non-existent session
2. **Invalid Rate**: Setting rate of 0 or negative value
3. **Missing Parameters**: Required fields not provided
4. **Duplicate Sessions**: Same IMSI connecting multiple times

### Error Recovery

The system provides graceful error handling:
- Failed billing attempts are logged but don't break sessions
- Network disconnections are handled automatically
- Rate changes apply to new sessions only

## 🔮 Future Enhancements

### Planned Features

1. **Quality of Service (QoS) Integration**
   - Different rates for different service levels
   - Bandwidth-based billing

2. **Fraud Detection**
   - Unusual usage pattern detection
   - Automatic session suspension

3. **Multi-Currency Support**
   - Different billing currencies
   - Real-time exchange rate integration

4. **Advanced Analytics**
   - Predictive billing
   - Usage trend analysis

## 🤝 Contributing

The GSM Roaming system is part of the distli-bc-mesh project. Contributions welcome:

1. **Smart Contract Enhancements**: Add new billing models
2. **UI Improvements**: Better visualization of roaming data  
3. **Integration Modules**: Connect with real network equipment
4. **Testing**: More comprehensive test scenarios

---

*Generated with ❤️ by the distli-bc-mesh GSM Roaming Smart Contract System*
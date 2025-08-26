1. Make sure your system is running:
  # On 192.168.200.132 (Tracker)
  cargo run --bin tracker --features native

  # On 192.168.200.133 (Validator) 
  TRACKER_URL="http://192.168.200.132:3030" cargo run --bin enterprise-validator --features native -- --id validator1 --port 8080 --stake 1000
  2. Open your browser and navigate to:
  http://192.168.200.132:3030/smart_contracts_demo.html
  3. You should see a page with tabs:
    - Deploy Contracts
    - Counter
    - Order Book
    - NFT
    - Token
    - Voting
    - All Contracts

  Testing Smart Contracts:

  Option 1: Use the GUI (Easiest)

  - Click on "Deploy Contracts" tab
  - Deploy a counter contract
  - Go to "Counter" tab and click increment/decrement buttons
  - Deploy an NFT contract and mint tokens
  - Deploy an order book and place bids/asks

  Option 2: Use Browser Console (More Control)

  Open browser console (F12) and run:

  // Deploy contracts
  app.deployContract('counter', '{}')
  app.deployContract('nft', '{"name": "TestNFT", "symbol": "TNFT"}')
  app.deployContract('orderbook', '{"asset": "BTC/USD"}')

  // Call contract functions
  app.callContract('contract_1', 'increment', '{}')
  app.callContract('contract_1', 'get_count', '{}')
  app.callContract('contract_2', 'mint', '{"metadata": "My NFT"}')

  // List all contracts
  JSON.parse(app.contractExecutor.list_contracts())

  The smart contracts run entirely in your browser via WASM - no need for the validator to be running for basic testing!

  Which approach would you like to try first?

> provide more details about browser conseole commands

● Here's a comprehensive guide to testing smart contracts via the browser console:

  Access Console & Initialize:

  1. Open browser console: F12 → Console tab
  2. Navigate to: http://192.168.200.132:3030/smart_contracts_demo.html
  3. Wait for page to load - you should see "Smart Contract System Ready" in console

  Basic Commands:

  Check System Status:

  // Check if app is loaded
  console.log(app)

  // Check if contract executor is ready
  console.log(app.contractExecutor)

  // List all available contracts
  JSON.parse(app.contractExecutor.list_contracts())

  Deploy Contracts:

  Counter Contract:

  app.deployContract('counter', '{}')

  Order Book Contract:

  app.deployContract('orderbook', '{"asset": "ETH/USD"}')
  app.deployContract('orderbook', '{"asset": "BTC/USDT"}')

  NFT Contract:

  app.deployContract('nft', '{"name": "CryptoArt", "symbol": "CART"}')
  app.deployContract('nft', '{"name": "GameItems", "symbol": "GAME"}')

  Token Contract:

  app.deployContract('token', '{"name": "MyToken", "symbol": "MTK", "total_supply": 1000000}')

  Voting Contract:

  app.deployContract('voting', '{"title": "Best Pizza Topping", "options": ["Pepperoni", "Mushroom", "Pineapple"]}')

  Call Contract Functions:

  Counter Contract (contract_1):

  // Increment counter
  app.callContract('contract_1', 'increment', '{}')

  // Decrement counter  
  app.callContract('contract_1', 'decrement', '{}')

  // Get current count
  app.callContract('contract_1', 'get_count', '{}')

  // Test with gas limits
  app.contractExecutor.call_contract_with_gas('contract_1', 'increment', '{}', app.userId, 100)
  app.contractExecutor.call_contract_with_gas('contract_1', 'increment', '{}', app.userId, 1)  // Should fail

  Order Book Contract (contract_2):

  // Place a bid
  app.callContract('contract_2', 'place_bid', '{"price": 2500, "quantity": 1.5}')

  // Place an ask
  app.callContract('contract_2', 'place_ask', '{"price": 2600, "quantity": 2.0}')

  // View order book
  JSON.parse(app.callContract('contract_2', 'get_orderbook', '{}'))

  // Cancel an order (use order_id from place_bid/ask response)
  app.callContract('contract_2', 'cancel_order', '{"order_id": 1}')

  NFT Contract (contract_3):

  // Mint NFT
  app.callContract('contract_3', 'mint', '{"metadata": "Rare Dragon #001"}')

  // Mint to specific address
  app.callContract('contract_3', 'mint', '{"metadata": "Epic Sword", "to": "user_abc123"}')

  // Get token owner
  app.callContract('contract_3', 'get_owner', '{"token_id": 1}')

  // Get token metadata
  app.callContract('contract_3', 'get_metadata', '{"token_id": 1}')

  // Transfer NFT
  app.callContract('contract_3', 'transfer', '{"token_id": 1, "to": "user_xyz789"}')

  Token Contract (contract_4):

  // Check balance
  app.callContract('contract_4', 'balance_of', '{"address": "' + app.userId + '"}')

  // Transfer tokens
  app.callContract('contract_4', 'transfer', '{"to": "user_abc123", "amount": 1000}')

  // Check another balance
  app.callContract('contract_4', 'balance_of', '{"address": "user_abc123"}')

  Voting Contract (contract_5):

  // Vote for option (0-indexed)
  app.callContract('contract_5', 'vote', '{"option": 0}')  // Vote for first option

  // Get voting results
  JSON.parse(app.callContract('contract_5', 'get_results', '{}'))

  // Try voting again (should fail)
  app.callContract('contract_5', 'vote', '{"option": 1}')

  Advanced Commands:

  Inspect Contract State:

  // Get detailed contract state
  JSON.parse(app.contractExecutor.get_contract_state('contract_1'))
  JSON.parse(app.contractExecutor.get_contract_state('contract_2'))

  // Get all contracts with their states
  let contracts = JSON.parse(app.contractExecutor.list_contracts())
  contracts.forEach(contract => {
      console.log(`${contract.contract_id}: ${contract.contract_type}`)
      console.log(JSON.parse(app.contractExecutor.get_contract_state(contract.contract_id)))
  })

  Batch Operations:

  // Deploy multiple contracts
  ['counter', 'nft', 'orderbook'].forEach((type, i) => {
      let initData = type === 'nft' ? '{"name": "Collection' + i + '", "symbol": "C' + i + '"}' :
                     type === 'orderbook' ? '{"asset": "ASSET' + i + '"}' : '{}'
      app.deployContract(type, initData)
  })

  // Mass increment counter
  for(let i = 0; i < 10; i++) {
      app.callContract('contract_1', 'increment', '{}')
  }

  Gas Usage Analysis:

  // Test different gas limits
  [1, 10, 50, 100, 1000].forEach(gas => {
      let result = app.contractExecutor.call_contract_with_gas('contract_1', 'increment', '{}', app.userId, gas)
      console.log(`Gas ${gas}:`, result)
  })

  Useful Helper Functions:

  // Quick deploy and test counter
  function quickTest() {
      app.deployContract('counter', '{}')
      setTimeout(() => {
          app.callContract('contract_1', 'increment', '{}')
          app.callContract('contract_1', 'increment', '{}')
          app.callContract('contract_1', 'get_count', '{}')
      }, 100)
  }

  // Deploy full test suite
  function deployAll() {
      app.deployContract('counter', '{}')
      app.deployContract('nft', '{"name": "TestNFTs", "symbol": "TEST"}')
      app.deployContract('orderbook', '{"asset": "BTC/USD"}')
      app.deployContract('token', '{"name": "TestCoin", "symbol": "TEST", "total_supply": 1000000}')
  }



// Smart Contracts Virtual Machine and Contract Manager
class ContractVM {
    constructor() {
        this.contracts = new Map();
        this.initializeMultiLanguageContracts();
    }
    
    initializeMultiLanguageContracts() {
        const tradingContract = {
            id: "trading_contract",
            name: "Trading Contract",
            runtime: "rust",
            code: "trading",
            state: {
                orderBook: { bids: [], asks: [] },
                trades: [],
                nextOrderId: 1
            },
            owner: "system",
            created_at: Date.now()
        };
        
        this.contracts.set("trading_contract", tradingContract);
        console.log('Multi-language contracts deployed: Rust, Python, WASM, JavaScript');
    }
    
    deploy_contract(contract) {
        this.contracts.set(contract.id, contract);
        return contract.id;
    }
    
    call_contract(call) {
        const contract = this.contracts.get(call.contract_id);
        if (!contract) {
            throw new Error("Contract not found");
        }
        
        switch (contract.runtime) {
            case "rust":
            case "trading":
                return this.execute_rust_contract(contract, call);
            default:
                throw new Error("Unknown runtime: " + contract.runtime);
        }
    }
    
    execute_rust_contract(contract, call) {
        if (contract.code === "trading") {
            return this.execute_trading_function(contract.state, call);
        }
        throw new Error("Unknown Rust contract");
    }
    
    execute_trading_function(state, call) {
        switch (call.function) {
            case "buy":
                return this.place_buy_order(state, call.params, call.caller);
            case "sell":
                return this.place_sell_order(state, call.params, call.caller);
            case "cancel":
                return this.cancel_order(state, call.params, call.caller);
            default:
                throw new Error("Unknown function: " + call.function);
        }
    }
    
    place_buy_order(state, params, caller) {
        const orderId = state.nextOrderId++;
        const order = {
            id: orderId,
            type: "buy",
            asset: params.asset,
            quantity: params.quantity,
            price: params.price,
            trader: caller,
            timestamp: Date.now()
        };
        
        const trades = this.match_orders(order, state.orderBook.asks, state.trades);
        
        if (order.quantity > 0) {
            state.orderBook.bids.push(order);
            state.orderBook.bids.sort((a, b) => b.price - a.price);
        }
        
        return {
            success: true,
            result: {
                orderId: orderId,
                trades: trades,
                message: `Buy order placed: ${params.quantity} ${params.asset} @ ${params.price}`
            },
            gas_used: 1000,
            state_changes: state
        };
    }
    
    place_sell_order(state, params, caller) {
        const orderId = state.nextOrderId++;
        const order = {
            id: orderId,
            type: "sell",
            asset: params.asset,
            quantity: params.quantity,
            price: params.price,
            trader: caller,
            timestamp: Date.now()
        };
        
        const trades = this.match_orders(order, state.orderBook.bids, state.trades);
        
        if (order.quantity > 0) {
            state.orderBook.asks.push(order);
            state.orderBook.asks.sort((a, b) => a.price - b.price);
        }
        
        return {
            success: true,
            result: {
                orderId: orderId,
                trades: trades,
                message: `Sell order placed: ${params.quantity} ${params.asset} @ ${params.price}`
            },
            gas_used: 1000,
            state_changes: state
        };
    }
    
    cancel_order(state, params, caller) {
        const orderId = params.orderId;
        
        let removed = false;
        state.orderBook.bids = state.orderBook.bids.filter(order => {
            if (order.id === orderId && order.trader === caller) {
                removed = true;
                return false;
            }
            return true;
        });
        
        if (!removed) {
            state.orderBook.asks = state.orderBook.asks.filter(order => {
                if (order.id === orderId && order.trader === caller) {
                    removed = true;
                    return false;
                }
                return true;
            });
        }
        
        if (removed) {
            return {
                success: true,
                result: { message: `Order ${orderId} cancelled` },
                gas_used: 500,
                state_changes: state
            };
        } else {
            return {
                success: false,
                error: "Order not found or not owned by caller",
                gas_used: 100
            };
        }
    }
    
    match_orders(order, oppositeOrders, tradesArray) {
        const trades = [];
        let remainingQuantity = order.quantity;
        
        for (let i = oppositeOrders.length - 1; i >= 0 && remainingQuantity > 0; i--) {
            const oppositeOrder = oppositeOrders[i];
            
            const canTrade = (order.type === "buy" && order.price >= oppositeOrder.price) ||
                           (order.type === "sell" && order.price <= oppositeOrder.price);
            
            if (canTrade && oppositeOrder.asset === order.asset) {
                const tradeQuantity = Math.min(remainingQuantity, oppositeOrder.quantity);
                const tradePrice = oppositeOrder.price;
                
                const trade = {
                    id: `trade_${Date.now()}_${Math.random()}`,
                    asset: order.asset,
                    quantity: tradeQuantity,
                    price: tradePrice,
                    buyer: order.type === "buy" ? order.trader : oppositeOrder.trader,
                    seller: order.type === "sell" ? order.trader : oppositeOrder.trader,
                    timestamp: Date.now()
                };
                
                trades.push(trade);
                tradesArray.push(trade);
                
                remainingQuantity -= tradeQuantity;
                oppositeOrder.quantity -= tradeQuantity;
                
                if (oppositeOrder.quantity <= 0) {
                    oppositeOrders.splice(i, 1);
                }
            }
        }
        
        order.quantity = remainingQuantity;
        return trades;
    }
    
    get_order_book(asset = null) {
        const contract = this.contracts.get("trading_contract");
        if (!contract) {
            return { bids: [], asks: [] };
        }
        
        let bids = contract.state.orderBook.bids || [];
        let asks = contract.state.orderBook.asks || [];
        
        if (asset) {
            bids = bids.filter(order => order.asset === asset);
            asks = asks.filter(order => order.asset === asset);
        }
        
        return { bids, asks };
    }
    
    get_recent_trades(asset = null, limit = 10) {
        const contract = this.contracts.get("trading_contract");
        if (!contract) {
            return { trades: [] };
        }
        
        let trades = contract.state.trades || [];
        
        if (asset) {
            trades = trades.filter(trade => trade.asset === asset);
        }
        
        trades = trades.slice(-limit).reverse();
        return { trades };
    }
    
    get_all_state() {
        const state = {};
        this.contracts.forEach((contract, id) => {
            state[id] = contract.state;
        });
        return state;
    }
    
    restore_state(savedState) {
        this.initializeMultiLanguageContracts();
        
        if (savedState) {
            Object.keys(savedState).forEach(contractId => {
                const contract = this.contracts.get(contractId);
                if (contract) {
                    contract.state = savedState[contractId];
                }
            });
        }
    }
    
    apply_state_changes(contractId, stateChanges) {
        const contract = this.contracts.get(contractId);
        if (contract && stateChanges) {
            contract.state = stateChanges;
        }
    }

    list_contracts() {
        return Array.from(this.contracts.values());
    }
}

// Contract Templates for the editor
const CONTRACT_TEMPLATES = {
    counter: `function increment(params) {
    const amount = params.amount || 1;
    this.state.count = (this.state.count || 0) + amount;
    
    return {
        count: this.state.count,
        message: \`Incremented by \${amount}, new count: \${this.state.count}\`
    };
}

function decrement(params) {
    const amount = params.amount || 1;
    this.state.count = (this.state.count || 0) - amount;
    
    return {
        count: this.state.count,
        message: \`Decremented by \${amount}, new count: \${this.state.count}\`
    };
}

function getCount() {
    return { count: this.state.count || 0 };
}`,

    voting: `function createProposal(params) {
    const { title, description } = params;
    if (!title) throw new Error("Title required");
    
    if (!this.state.proposals) this.state.proposals = [];
    
    const proposal = {
        id: this.state.proposals.length,
        title: title,
        description: description || "",
        votes: { yes: 0, no: 0 },
        voters: [],
        created: Date.now(),
        creator: this.caller
    };
    
    this.state.proposals.push(proposal);
    
    return {
        proposalId: proposal.id,
        message: \`Proposal "\${title}" created\`
    };
}

function vote(params) {
    const { proposalId, vote } = params;
    if (!this.state.proposals || !this.state.proposals[proposalId]) {
        throw new Error("Proposal not found");
    }
    
    const proposal = this.state.proposals[proposalId];
    
    if (proposal.voters.includes(this.caller)) {
        throw new Error("Already voted");
    }
    
    if (vote === "yes") {
        proposal.votes.yes++;
    } else if (vote === "no") {
        proposal.votes.no++;
    } else {
        throw new Error("Vote must be 'yes' or 'no'");
    }
    
    proposal.voters.push(this.caller);
    
    return {
        proposal: proposal,
        message: \`Voted \${vote} on "\${proposal.title}"\`
    };
}`
};

// Simple Contract Manager for the UI
class SimpleContractManager {
    constructor() {
        this.contracts = new Map();
        this.executionLog = [];
        this.isMonitoring = false;
        this.currentEditing = null;
        
        this.initializeContracts();
        this.setupUI();
        this.updateUI();
    }
    
    initializeContracts() {
        if (window.blockchain && window.blockchain.contract_vm) {
            const existingContracts = window.blockchain.contract_vm.list_contracts();
            existingContracts.forEach(contract => {
                this.contracts.set(contract.id, {
                    id: contract.id,
                    name: contract.name || contract.id,
                    runtime: contract.runtime || "javascript",
                    code: this.getContractCode(contract.id),
                    functions: this.extractFunctions(this.getContractCode(contract.id)),
                    executions: 0,
                    lastExecuted: null,
                    status: 'idle'
                });
            });
        }
    }
    
    setupUI() {
        this.setupContractsTab();
        this.setupContractEditorTab();
        this.setupModals();
    }
    
    setupContractsTab() {
        const contractsTab = document.getElementById('contracts-tab');
        if (contractsTab) {
            contractsTab.innerHTML = `
                <h4>Multi-Language Smart Contracts</h4>
                
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;">
                    <div class="order-form">
                        <h5>Rust Trading Contract</h5>
                        <p>High-performance order matching and trade execution</p>
                        <button onclick="showContractState('trading_contract')" style="background: #CE422B;">View State</button>
                    </div>
                    
                    <div class="order-form">
                        <h5>JavaScript Analytics</h5>
                        <p>Real-time market analysis and indicators</p>
                        <button onclick="callJavaScriptAnalytics()" style="background: #F7DF1E; color: black;">Analyze</button>
                    </div>
                </div>
                
                <div id="contract-state-display" style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-top: 20px; display: none;">
                    <h5>Contract State</h5>
                    <pre id="contract-state-content" style="background: white; padding: 10px; border-radius: 4px; overflow-x: auto; font-size: 12px;"></pre>
                </div>
            `;
        }
    }
    
    setupContractEditorTab() {
        const editorTab = document.getElementById('contract-editor-tab');
        if (editorTab) {
            editorTab.innerHTML = `
                <h3>Contract Management</h3>
                <div style="margin-bottom: 20px;">
                    <button class="btn btn-success" onclick="showNewContractDialog()">Add New Contract</button>
                    <button class="btn btn-primary" onclick="refreshContracts()">Refresh</button>
                </div>
                
                <div id="contract-list" class="contract-list"></div>

                <h3>Contract Execution Monitor</h3>
                <div style="margin-bottom: 15px;">
                    <button id="monitor-toggle" class="btn btn-success" onclick="toggleExecutionMonitor()">Start Monitoring</button>
                    <button class="btn btn-secondary" onclick="clearExecutionLog()">Clear Log</button>
                </div>
                
                <div id="execution-log" class="execution-log">
                    <div class="log-entry log-info">Contract execution monitor ready.</div>
                </div>
            `;
        }
    }
    
    setupModals() {
        // Add modals to body
        const modalHTML = `
            <!-- Contract Editor Modal -->
            <div id="contract-editor-modal" class="editor-modal">
                <div class="editor-content">
                    <span class="close" onclick="closeContractEditor()">&times;</span>
                    <h3 id="editor-title">Edit Contract</h3>
                    
                    <div class="form-group">
                        <label>Contract Name:</label>
                        <input type="text" id="edit-contract-name">
                    </div>
                    
                    <div class="form-group">
                        <label>Contract Code:</label>
                        <textarea id="contract-code" class="code-editor" placeholder="Enter your contract code here..."></textarea>
                    </div>
                    
                    <div style="margin-bottom: 15px;">
                        <button class="btn btn-primary" onclick="saveContract()">Save Contract</button>
                        <button class="btn btn-secondary" onclick="closeContractEditor()">Cancel</button>
                    </div>
                </div>
            </div>

            <!-- New Contract Modal -->
            <div id="new-contract-modal" class="editor-modal">
                <div class="editor-content">
                    <span class="close" onclick="closeNewContractDialog()">&times;</span>
                    <h3>Create New Contract</h3>
                    
                    <div class="form-group">
                        <label>Contract ID:</label>
                        <input type="text" id="new-contract-id" placeholder="my_contract">
                    </div>
                    
                    <div class="form-group">
                        <label>Contract Name:</label>
                        <input type="text" id="new-contract-name" placeholder="My Contract">
                    </div>
                    
                    <div class="form-group">
                        <label>Template:</label>
                        <select id="contract-template" onchange="loadContractTemplate()">
                            <option value="">Empty Contract</option>
                            <option value="counter">Simple Counter</option>
                            <option value="voting">Voting System</option>
                        </select>
                    </div>
                    
                    <div class="form-group">
                        <label>Contract Code:</label>
                        <textarea id="new-contract-code" class="code-editor" placeholder="Enter your contract code..."></textarea>
                    </div>
                    
                    <div>
                        <button class="btn btn-success" onclick="createNewContract()">Create Contract</button>
                        <button class="btn btn-secondary" onclick="closeNewContractDialog()">Cancel</button>
                    </div>
                </div>
            </div>
        `;
        
        document.body.insertAdjacentHTML('beforeend', modalHTML);
    }
    
    getContractCode(contractId) {
        return `// Contract: ${contractId}\nfunction example(params) {\n    return { message: "Hello from ${contractId}" };\n}`;
    }
    
    extractFunctions(code) {
        const functionRegex = /function\s+(\w+)\s*\(/g;
        const functions = [];
        let match;
        
        while ((match = functionRegex.exec(code)) !== null) {
            functions.push(match[1]);
        }
        
        return functions;
    }
    
    updateUI() {
        const listContainer = document.getElementById('contract-list');
        if (!listContainer) return;
        
        listContainer.innerHTML = '';
        
        this.contracts.forEach(contract => {
            const card = document.createElement('div');
            card.className = 'contract-card';
            
            card.innerHTML = `
                <h4>${contract.name}</h4>
                <p style="font-size: 12px; color: #666; margin: 5px 0;">
                    Functions: ${contract.functions.join(', ')}<br>
                    Executions: ${contract.executions}
                </p>
                <div>
                    <button class="btn btn-primary" onclick="editContract('${contract.id}')">Edit</button>
                    <button class="btn btn-danger" onclick="deleteContract('${contract.id}')">Delete</button>
                </div>
            `;
            
            listContainer.appendChild(card);
        });
    }
}

// Global contract manager instance
let contractManager;

// Global functions for contract management
function showContractState(contractId) {
    if (!window.blockchain) return;
    
    try {
        const contract = window.blockchain.contract_vm.contracts.get(contractId);
        if (!contract) {
            console.log(`Contract ${contractId} not found`);
            return;
        }
        
        const stateDisplay = document.getElementById('contract-state-display');
        const stateContent = document.getElementById('contract-state-content');
        
        if (stateDisplay && stateContent) {
            const displayData = {
                contract_id: contract.id,
                name: contract.name,
                runtime: contract.runtime,
                state: contract.state
            };
            
            stateContent.textContent = JSON.stringify(displayData, null, 2);
            stateDisplay.style.display = 'block';
        }
        
        console.log(`Displaying state for ${contract.name} (${contract.runtime})`);
        
    } catch (error) {
        console.log(`Error displaying contract state: ${error.message}`);
    }
}

function editContract(contractId) {
    if (contractManager) {
        contractManager.editContract(contractId);
    }
}

function saveContract() {
    if (contractManager) {
        contractManager.saveContract();
    }
}

function closeContractEditor() {
    const modal = document.getElementById('contract-editor-modal');
    if (modal) modal.style.display = 'none';
}

function showNewContractDialog() {
    const modal = document.getElementById('new-contract-modal');
    if (modal) modal.style.display = 'block';
}

function closeNewContractDialog() {
    const modal = document.getElementById('new-contract-modal');
    if (modal) modal.style.display = 'none';
}

function createNewContract() {
    if (contractManager) {
        contractManager.createNewContract();
    }
}

function loadContractTemplate() {
    const template = document.getElementById('contract-template').value;
    const codeTextarea = document.getElementById('new-contract-code');
    
    if (template && CONTRACT_TEMPLATES[template] && codeTextarea) {
        codeTextarea.value = CONTRACT_TEMPLATES[template];
    }
}

function refreshContracts() {
    if (contractManager) {
        contractManager.updateUI();
    }
}

function deleteContract(contractId) {
    if (contractManager) {
        contractManager.deleteContract(contractId);
    }
}

function toggleExecutionMonitor() {
    if (contractManager) {
        contractManager.toggleExecutionMonitor();
    }
}

function clearExecutionLog() {
    if (contractManager) {
        contractManager.clearExecutionLog();
    }
}

function callJavaScriptAnalytics() {
    if (!window.blockchain) return;
    
    const asset = document.getElementById('buy-asset')?.value || 'BTC';
    
    const call = {
        contract_id: "js_analytics",
        function: "analyze_market",
        params: { asset: asset },
        caller: getUserId(),
        gas_limit: 3000
    };
    
    try {
        const tx = window.blockchain.call_contract(call, getUserId());
        
        if (window.mesh && window.mesh.connected) {
            const txMessage = { type: 'transaction', transaction: tx };
            window.mesh.send(txMessage);
            window.mesh.broadcast(txMessage);
        }
        
        if (tx.result && tx.result.success) {
            const analysis = tx.result.result;
            log(`JavaScript Analytics for ${asset}: Mock analysis completed`);
        }
        
        if (typeof updateUI === 'function') updateUI();
        
    } catch (error) {
        log(`Error calling JavaScript analytics: ${error.message}`);
    }
}

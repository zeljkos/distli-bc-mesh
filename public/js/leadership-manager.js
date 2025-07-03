// Enhanced Distributed Leadership Manager with GUI Integration
// File: public/js/leadership-manager.js

class DistributedLeadershipManager {
    constructor(blockchain, mesh) {
        this.blockchain = blockchain;
        this.mesh = mesh;
        this.nodeId = getUserId();
        
        // Leadership state
        this.currentLeader = null;
        this.leadershipTerm = 0;
        this.lastHeartbeat = new Map(); // nodeId -> timestamp
        this.leadershipVotes = new Map(); // term -> votes
        this.state = 'FOLLOWER'; // FOLLOWER, CANDIDATE, LEADER
        
        // Timers and intervals
        this.heartbeatInterval = null;
        this.electionTimeout = null;
        this.leaderVerificationInterval = null;
        
        // Configuration
        this.HEARTBEAT_INTERVAL = 3000; // 3 seconds
        this.ELECTION_TIMEOUT = 6000; // 6 seconds
        this.LEADER_TIMEOUT = 10000; // 10 seconds
        
        // GUI Integration
        this.onLeadershipChange = null; // Callback for UI updates
        this.lastUIUpdate = 0;
        
        // Performance tracking
        this.electionHistory = [];
        this.leadershipMetrics = {
            electionsStarted: 0,
            electionsWon: 0,
            totalLeadershipTime: 0,
            leadershipStartTime: null
        };
        
        // Initialize immediately
        this.initialize();
    }
    
    initialize() {
        console.log('🏗️ Initializing Enhanced Leadership System with GUI Integration');
        
        // Disable legacy blockchain leadership
        this.disableLegacyLeadership();
        
        // Setup message handling
        this.setupMessageHandlers();
        
        // Start monitoring
        this.startLeaderVerification();
        
        // Initial leadership determination (delayed to allow network setup)
        setTimeout(() => {
            this.checkInitialLeadership();
        }, 2000);
        
        // UI update interval
        setInterval(() => {
            this.updateUIIfNeeded();
        }, 1000);
    }
    
    disableLegacyLeadership() {
        console.log('🔇 Disabling legacy blockchain leadership methods');
        
        // Mark blockchain as using robust leadership
        this.blockchain.useRobustLeadership = true;
        
        // Override problematic legacy methods
        this.blockchain.checkLeadershipStatus = () => {
            // Delegate to this robust system
            this.blockchain.isMiningLeader = this.isLeader();
        };
        
        this.blockchain.becomeLeader = () => {
            console.log('🔇 Legacy becomeLeader disabled - using robust system');
        };
        
        this.blockchain.stepDownAsLeader = () => {
            console.log('🔇 Legacy stepDownAsLeader disabled - using robust system');
        };
        
        // Clear any existing legacy timers
        if (this.blockchain.leaderHeartbeatInterval) {
            clearInterval(this.blockchain.leaderHeartbeatInterval);
            this.blockchain.leaderHeartbeatInterval = null;
        }
        
        // Disable legacy startLeaderElection
        this.blockchain.startLeaderElection = () => {
            console.log('🔇 Legacy startLeaderElection disabled - using robust system');
        };
    }
    
    setupMessageHandlers() {
        // Store original message handler
        this.originalHandleMessage = this.mesh.handleMessage?.bind(this.mesh);
        
        // Override mesh message handling to intercept leadership messages
        const originalHandleMessage = this.mesh.handleMessage;
        this.mesh.handleMessage = (message) => {
            if (this.handleLeadershipMessage(message)) {
                return; // Leadership message handled
            }
            
            // Pass other messages to original handler
            if (originalHandleMessage) {
                return originalHandleMessage.call(this.mesh, message);
            }
        };
    }
    
    handleLeadershipMessage(message) {
        switch (message.type) {
            case 'leadership_heartbeat':
                return this.handleLeaderHeartbeat(message);
            case 'leadership_election':
                return this.handleElectionRequest(message);
            case 'leadership_vote':
                return this.handleVote(message);
            case 'leadership_announcement':
                return this.handleLeadershipAnnouncement(message);
            default:
                return false; // Not a leadership message
        }
    }
    
    // ============================================================================
    // LEADER ELECTION PROTOCOL
    // ============================================================================
    
    startElection() {
        console.log('🗳️ Starting enhanced leadership election');
        
        this.state = 'CANDIDATE';
        this.leadershipTerm++;
        this.currentLeader = null;
        this.leadershipMetrics.electionsStarted++;
        
        // Record election start
        this.electionHistory.push({
            term: this.leadershipTerm,
            startTime: Date.now(),
            outcome: 'pending',
            nodeId: this.nodeId
        });
        
        // Vote for self
        this.leadershipVotes.set(this.leadershipTerm, new Set([this.nodeId]));
        
        // Request votes from all peers
        this.broadcastElectionRequest();
        
        // Set election timeout
        this.resetElectionTimer();
        
        // Check results after timeout
        setTimeout(() => {
            this.checkElectionResults();
        }, 4000);
        
        this.triggerUIUpdate();
        log(`🗳️ Started election for term ${this.leadershipTerm}`);
    }
    
    broadcastElectionRequest() {
        const electionMessage = {
            type: 'leadership_election',
            candidateId: this.nodeId,
            term: this.leadershipTerm,
            timestamp: Date.now(),
            networkId: this.mesh.currentNetwork,
            metrics: this.getElectionMetrics()
        };
        
        console.log(`📢 Broadcasting election request for term ${this.leadershipTerm}`);
        this.safeBroadcast(electionMessage);
    }
    
    handleElectionRequest(message) {
        console.log(`🗳️ Received election request from ${message.candidateId.substring(0, 8)} for term ${message.term}`);
        
        // Only vote if this is a newer term or we haven't voted yet
        if (message.term > this.leadershipTerm || 
           (message.term === this.leadershipTerm && this.state === 'FOLLOWER')) {
            
            this.leadershipTerm = message.term;
            this.state = 'FOLLOWER';
            this.currentLeader = null;
            
            // Enhanced voting logic - consider candidate fitness
            if (this.shouldVoteForCandidate(message)) {
                this.sendVote(message.candidateId, message.term);
            }
            
            this.triggerUIUpdate();
        }
        
        return true;
    }
    
    shouldVoteForCandidate(electionMessage) {
        // Simple fitness check - could be enhanced with more criteria
        const candidateMetrics = electionMessage.metrics || {};
        
        // Basic validation
        if (!electionMessage.candidateId || !electionMessage.networkId) {
            return false;
        }
        
        // Check if candidate is from our network
        if (electionMessage.networkId !== this.mesh.currentNetwork) {
            return false;
        }
        
        // For now, vote yes (could add more sophisticated criteria)
        return true;
    }
    
    sendVote(candidateId, term) {
        const voteMessage = {
            type: 'leadership_vote',
            voterId: this.nodeId,
            candidateId: candidateId,
            term: term,
            timestamp: Date.now(),
            networkId: this.mesh.currentNetwork
        };
        
        console.log(`✅ Voting for ${candidateId.substring(0, 8)} in term ${term}`);
        this.safeBroadcast(voteMessage);
    }
    
    handleVote(message) {
        console.log(`🗳️ Received vote from ${message.voterId.substring(0, 8)} for ${message.candidateId.substring(0, 8)} in term ${message.term}`);
        
        // Only count votes for current term and for ourselves
        if (message.term === this.leadershipTerm && message.candidateId === this.nodeId) {
            if (!this.leadershipVotes.has(message.term)) {
                this.leadershipVotes.set(message.term, new Set());
            }
            this.leadershipVotes.get(message.term).add(message.voterId);
            
            // Check if we have majority
            setTimeout(() => this.checkElectionResults(), 500);
        }
        
        return true;
    }
    
    checkElectionResults() {
        if (this.state !== 'CANDIDATE') return;
        
        const votes = this.leadershipVotes.get(this.leadershipTerm);
        const totalPeers = this.getTotalPeerCount();
        const majority = Math.floor(totalPeers / 2) + 1;
        
        console.log(`📊 Election results: ${votes ? votes.size : 0}/${totalPeers} votes (need ${majority})`);
        
        // Update election history
        const election = this.electionHistory.find(e => e.term === this.leadershipTerm);
        if (election) {
            election.votesReceived = votes ? votes.size : 0;
            election.totalPeers = totalPeers;
            election.majorityNeeded = majority;
        }
        
        if (votes && votes.size >= majority) {
            this.becomeLeader();
            if (election) election.outcome = 'won';
        } else if (this.state === 'CANDIDATE') {
            console.log('❌ Election failed, becoming follower');
            this.becomeFollower();
            if (election) election.outcome = 'lost';
        }
    }
    
    // ============================================================================
    // LEADERSHIP MANAGEMENT
    // ============================================================================
    
    becomeLeader() {
        console.log('👑 BECOMING VERIFIED NETWORK LEADER');
        
        this.state = 'LEADER';
        this.currentLeader = this.nodeId;
        this.blockchain.isMiningLeader = true;
        this.leadershipMetrics.electionsWon++;
        this.leadershipMetrics.leadershipStartTime = Date.now();
        
        // Clear election timer
        this.clearElectionTimer();
        
        // Start sending heartbeats
        this.startHeartbeats();
        
        // Announce leadership
        this.announceLeadership();
        
        // Update UI
        this.triggerUIUpdate();
        
        // Auto-mine any pending transactions
        this.checkForAutoMining();
        
        log(`👑 Became verified leader for network ${this.mesh.currentNetwork}`);
    }
    
    becomeFollower(leaderId = null) {
        const wasLeader = this.state === 'LEADER';
        
        console.log(`👥 BECOMING FOLLOWER${leaderId ? ` (leader: ${leaderId.substring(0, 8)})` : ''}`);
        
        // Update leadership metrics if stepping down from leader
        if (wasLeader && this.leadershipMetrics.leadershipStartTime) {
            this.leadershipMetrics.totalLeadershipTime += 
                Date.now() - this.leadershipMetrics.leadershipStartTime;
            this.leadershipMetrics.leadershipStartTime = null;
        }
        
        this.state = 'FOLLOWER';
        this.currentLeader = leaderId;
        this.blockchain.isMiningLeader = false;
        
        // Stop heartbeats
        this.stopHeartbeats();
        
        // Start election timer
        this.resetElectionTimer();
        
        // Update UI
        this.triggerUIUpdate();
        
        if (wasLeader) {
            log(`Stepped down as leader for network ${this.mesh.currentNetwork}`);
        } else {
            log(`Following leader: ${leaderId ? leaderId.substring(0, 8) : 'TBD'}`);
        }
    }
    
    // ============================================================================
    // HEARTBEAT SYSTEM
    // ============================================================================
    
    startHeartbeats() {
        this.stopHeartbeats(); // Clear any existing
        
        this.heartbeatInterval = setInterval(() => {
            if (this.state === 'LEADER') {
                this.sendHeartbeat();
            }
        }, this.HEARTBEAT_INTERVAL);
        
        // Send immediate heartbeat
        this.sendHeartbeat();
    }
    
    stopHeartbeats() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }
    
    sendHeartbeat() {
        const heartbeatMessage = {
            type: 'leadership_heartbeat',
            leaderId: this.nodeId,
            term: this.leadershipTerm,
            timestamp: Date.now(),
            networkId: this.mesh.currentNetwork,
            pendingTransactions: this.blockchain.pending.length,
            blockHeight: this.blockchain.chain.length,
            metrics: this.getLeadershipMetrics()
        };
        
        console.log('💓 Sending enhanced leader heartbeat');
        this.safeBroadcast(heartbeatMessage);
    }
    
    handleLeaderHeartbeat(message) {
        console.log(`💓 Received heartbeat from leader ${message.leaderId.substring(0, 8)}`);
        
        // Update leader information
        this.lastHeartbeat.set(message.leaderId, message.timestamp);
        
        // If this is from a valid leader with newer or equal term
        if (message.term >= this.leadershipTerm) {
            if (this.state !== 'FOLLOWER' || this.currentLeader !== message.leaderId) {
                this.becomeFollower(message.leaderId);
            }
            this.leadershipTerm = message.term;
            this.currentLeader = message.leaderId;
            
            // Reset election timer since we heard from leader
            this.resetElectionTimer();
            
            // Trigger UI update
            this.triggerUIUpdate();
        }
        
        return true;
    }
    
    // ============================================================================
    // LEADERSHIP VERIFICATION & MONITORING
    // ============================================================================
    
    startLeaderVerification() {
        this.leaderVerificationInterval = setInterval(() => {
            this.verifyLeadershipStatus();
        }, 4000);
    }
    
    verifyLeadershipStatus() {
        const now = Date.now();
        
        // Check if current leader is still active
        if (this.currentLeader && this.currentLeader !== this.nodeId) {
            const lastSeen = this.lastHeartbeat.get(this.currentLeader);
            
            if (!lastSeen || (now - lastSeen) > this.LEADER_TIMEOUT) {
                console.log(`⚠️ Leader ${this.currentLeader.substring(0, 8)} appears offline, starting election`);
                this.currentLeader = null;
                this.startElection();
                return;
            }
        }
        
        // If we're not in an election and have no leader, start election
        if (!this.currentLeader && this.state === 'FOLLOWER') {
            console.log('🔍 No leader detected, starting election');
            this.startElection();
            return;
        }
        
        // Auto-mine if we're leader with pending transactions
        if (this.state === 'LEADER') {
            this.checkForAutoMining();
        }
        
        // Update blockchain leadership flag
        this.blockchain.isMiningLeader = this.isLeader();
        
        // Periodic UI update
        this.triggerUIUpdate();
    }
    
    checkForAutoMining() {
        if (this.state === 'LEADER' && this.blockchain.pending.length > 0) {
            console.log(`⛏️ Auto-mining ${this.blockchain.pending.length} pending transactions as verified leader`);
            this.mineAsLeader();
        }
    }
    
    mineAsLeader() {
        try {
            const block = this.blockchain.mineBlock();
            if (block) {
                console.log(`✅ Leader mined block #${block.id}`);
                
                // Broadcast block to network
                if (this.blockchain.broadcastNewBlock) {
                    this.blockchain.broadcastNewBlock(block);
                } else if (this.mesh && this.mesh.broadcast) {
                    this.mesh.broadcast({ type: 'block', block: block });
                }
                
                // Update UI
                this.triggerUIUpdate();
                if (typeof updateOrderBookFromContract === 'function') {
                    updateOrderBookFromContract();
                }
                
                log(`⛏️ Block #${block.id} mined by verified leader`);
            }
        } catch (error) {
            console.error('❌ Leader mining error:', error);
        }
    }
    
    // ============================================================================
    // GUI INTEGRATION
    // ============================================================================
    
    triggerUIUpdate() {
        // Rate limit UI updates
        const now = Date.now();
        if (now - this.lastUIUpdate < 500) return;
        this.lastUIUpdate = now;
        
        // Call the UI update callback
        if (this.onLeadershipChange && typeof this.onLeadershipChange === 'function') {
            this.onLeadershipChange(this.isLeader(), this.currentLeader);
        }
        
        // Update main UI
        if (typeof updateUI === 'function') {
            updateUI();
        }
    }
    
    updateUIIfNeeded() {
        // Periodic check for UI updates (low frequency)
        const now = Date.now();
        if (now - this.lastUIUpdate > 5000) { // Update every 5 seconds at minimum
            this.triggerUIUpdate();
        }
    }
    
    // ============================================================================
    // UTILITY METHODS
    // ============================================================================
    
    getTotalPeerCount() {
        // Include self + connected peers
        return 1 + (this.mesh.dataChannels ? this.mesh.dataChannels.size : 0);
    }
    
    checkInitialLeadership() {
        const peerCount = this.getTotalPeerCount();
        
        if (peerCount === 1) {
            // Solo node - become leader immediately
            console.log('🔧 Solo node detected - becoming immediate leader');
            this.leadershipTerm = 1;
            this.becomeLeader();
        } else {
            // Multiple peers - start election process with random delay
            console.log(`🔍 ${peerCount} peers detected - starting leadership determination`);
            const delay = Math.random() * 3000; // 0-3 second random delay
            setTimeout(() => {
                this.startElection();
            }, delay);
        }
    }
    
    announceLeadership() {
        const announcement = {
            type: 'leadership_announcement',
            leaderId: this.nodeId,
            term: this.leadershipTerm,
            timestamp: Date.now(),
            networkId: this.mesh.currentNetwork,
            metrics: this.getLeadershipMetrics()
        };
        
        this.safeBroadcast(announcement);
    }
    
    handleLeadershipAnnouncement(message) {
        console.log(`📢 Leadership announcement from ${message.leaderId.substring(0, 8)}`);
        
        if (message.term >= this.leadershipTerm) {
            this.becomeFollower(message.leaderId);
            this.leadershipTerm = message.term;
        }
        
        return true;
    }
    
    safeBroadcast(message) {
        try {
            if (this.mesh && this.mesh.broadcast && this.mesh.connected) {
                this.mesh.broadcast(message);
            }
        } catch (error) {
            console.error('Failed to broadcast leadership message:', error);
        }
    }
    
    // ============================================================================
    // METRICS AND ANALYTICS
    // ============================================================================
    
    getElectionMetrics() {
        return {
            electionsStarted: this.leadershipMetrics.electionsStarted,
            electionsWon: this.leadershipMetrics.electionsWon,
            successRate: this.leadershipMetrics.electionsStarted > 0 ? 
                (this.leadershipMetrics.electionsWon / this.leadershipMetrics.electionsStarted) : 0,
            nodeId: this.nodeId.substring(0, 8)
        };
    }
    
    getLeadershipMetrics() {
        let currentLeadershipTime = 0;
        if (this.state === 'LEADER' && this.leadershipMetrics.leadershipStartTime) {
            currentLeadershipTime = Date.now() - this.leadershipMetrics.leadershipStartTime;
        }
        
        return {
            ...this.leadershipMetrics,
            currentLeadershipTime: currentLeadershipTime,
            averageLeadershipTime: this.leadershipMetrics.electionsWon > 0 ?
                (this.leadershipMetrics.totalLeadershipTime / this.leadershipMetrics.electionsWon) : 0,
            totalPeers: this.getTotalPeerCount(),
            lastHeartbeatCount: this.lastHeartbeat.size
        };
    }
    
    // ============================================================================
    // TIMER MANAGEMENT
    // ============================================================================
    
    resetElectionTimer() {
        this.clearElectionTimer();
        
        // Random election timeout to prevent split votes
        const timeout = this.ELECTION_TIMEOUT + (Math.random() * 3000);
        
        this.electionTimeout = setTimeout(() => {
            if (this.state === 'FOLLOWER' && !this.currentLeader) {
                console.log('⏰ Election timeout - starting election');
                this.startElection();
            }
        }, timeout);
    }
    
    clearElectionTimer() {
        if (this.electionTimeout) {
            clearTimeout(this.electionTimeout);
            this.electionTimeout = null;
        }
    }
    
    // ============================================================================
    // PUBLIC API
    // ============================================================================
    
    getCurrentLeader() {
        return this.currentLeader;
    }
    
    isLeader() {
        return this.state === 'LEADER';
    }
    
    getState() {
        return {
            state: this.state,
            currentLeader: this.currentLeader,
            term: this.leadershipTerm,
            totalPeers: this.getTotalPeerCount(),
            networkId: this.mesh?.currentNetwork,
            metrics: this.getLeadershipMetrics()
        };
    }
    
    forceElection() {
        console.log('🔧 Forcing new leadership election');
        this.startElection();
    }
    
    getDebugInfo() {
        return {
            nodeId: this.nodeId.substring(0, 8),
            state: this.state,
            currentLeader: this.currentLeader?.substring(0, 8),
            term: this.leadershipTerm,
            totalPeers: this.getTotalPeerCount(),
            networkId: this.mesh?.currentNetwork,
            lastHeartbeats: Object.fromEntries(
                Array.from(this.lastHeartbeat.entries()).map(([id, time]) => [
                    id.substring(0, 8), 
                    new Date(time).toLocaleTimeString()
                ])
            ),
            pendingTransactions: this.blockchain?.pending?.length || 0,
            electionHistory: this.electionHistory.slice(-5), // Last 5 elections
            metrics: this.getLeadershipMetrics()
        };
    }
    
    // Cleanup
    destroy() {
        console.log('🧹 Destroying leadership manager');
        
        this.stopHeartbeats();
        this.clearElectionTimer();
        
        if (this.leaderVerificationInterval) {
            clearInterval(this.leaderVerificationInterval);
        }
        
        // Update leadership metrics if stepping down
        if (this.state === 'LEADER' && this.leadershipMetrics.leadershipStartTime) {
            this.leadershipMetrics.totalLeadershipTime += 
                Date.now() - this.leadershipMetrics.leadershipStartTime;
        }
        
        // Restore original message handler
        if (this.originalHandleMessage) {
            this.mesh.handleMessage = this.originalHandleMessage;
        }
        
        // Clear blockchain leadership
        if (this.blockchain) {
            this.blockchain.isMiningLeader = false;
            this.blockchain.useRobustLeadership = false;
        }
    }
}

// Make the class globally available
window.DistributedLeadershipManager = DistributedLeadershipManager;

console.log('✅ Enhanced DistributedLeadershipManager loaded with GUI integration');

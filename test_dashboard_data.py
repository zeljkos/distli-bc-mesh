#!/usr/bin/env python3
"""
Test script to show what the ZK dashboard should display
based on the data in the validator's blockchain
"""

import json

def load_validator_data():
    with open('data/enterprise_blockchain_validator1.json', 'r') as f:
        return json.load(f)

def analyze_zk_data():
    data = load_validator_data()
    
    print("🔐 ZK Proof Dashboard Data Analysis")
    print("=" * 50)
    
    # Get tenant blocks
    tenant_blocks = data.get('tenant_blocks', [])
    zk_blocks = [b for b in tenant_blocks if b.get('network_id') == 'zk_real_proofs']
    
    print(f"\n📊 Found {len(zk_blocks)} ZK proof blocks")
    
    contracts = []
    sessions = []
    
    for block in zk_blocks:
        print(f"\nBlock {block.get('block_id')}: {block.get('block_hash')}")
        
        for tx_str in block.get('transactions', []):
            try:
                tx = json.loads(tx_str)
                content = tx.get('tx_type', {}).get('Message', {}).get('content', '')
                
                if 'ZK_CONTRACT' in content:
                    # Parse contract
                    parties = extract_value(content, 'PARTIES:')
                    proof_size = extract_value(content, 'DURATION_PROOF:')
                    commitment = extract_value(content, 'COMMITMENT:')
                    verified = 'VERIFIED:true' in content
                    
                    contracts.append({
                        'id': tx.get('id'),
                        'parties': parties,
                        'proof_size': proof_size,
                        'commitment': commitment,
                        'verified': verified,
                        'from_to': f"{tx.get('from')} ↔ {tx.get('to')}"
                    })
                    
                elif 'ZK_SESSION' in content:
                    # Parse session
                    duration = extract_value(content, 'DURATION:')
                    proof_type = extract_value(content, 'PROOF_TYPE:')
                    verified = 'VERIFIED:true' in content
                    
                    sessions.append({
                        'id': tx.get('id'),
                        'duration': duration,
                        'proof_type': proof_type,
                        'verified': verified
                    })
                    
            except json.JSONDecodeError:
                print(f"  Error parsing transaction: {tx_str[:50]}...")
    
    print(f"\n🤝 Contracts Found: {len(contracts)}")
    for contract in contracts:
        print(f"  • {contract['from_to']}")
        print(f"    ID: {contract['id']}")
        print(f"    Parties: {contract['parties']}")
        print(f"    Proof Size: {contract['proof_size']}")
        print(f"    Commitment: {contract['commitment']}")
        print(f"    Verified: {'✅' if contract['verified'] else '❌'}")
        print()
    
    print(f"📞 Sessions Found: {len(sessions)}")
    for session in sessions:
        print(f"  • Duration: {session['duration']} min")
        print(f"    Proof Type: {session['proof_type']}")
        print(f"    Verified: {'✅' if session['verified'] else '❌'}")
    
    print(f"\n🎯 Dashboard Display Summary:")
    print(f"  • Active Contracts: {len(contracts)}")
    print(f"  • ZK Proofs Generated: {len(sessions) + len(contracts)}")
    print(f"  • Proof Size: 672B (constant)")
    print(f"  • Verification Time: ~5ms")
    
    print(f"\n🔍 What operators can see:")
    operators = ['T-Mobile', 'Vodafone', 'Orange', 'AT&T']
    
    for op in operators:
        visible_contracts = [c for c in contracts if op in c['parties']]
        print(f"  {op}: Can decrypt {len(visible_contracts)} contract(s)")
        for contract in visible_contracts:
            print(f"    - {contract['parties']} (Rate: Decrypted)")
        
        encrypted_contracts = [c for c in contracts if op not in c['parties']]
        for contract in encrypted_contracts:
            print(f"    - {contract['parties']} (Rate: ENCRYPTED)")

def extract_value(content, key):
    """Extract value from ZK content string"""
    try:
        start = content.find(key) + len(key)
        end = content.find('|', start)
        if end == -1:
            end = len(content)
        return content[start:end].strip()
    except:
        return 'N/A'

if __name__ == '__main__':
    try:
        analyze_zk_data()
    except FileNotFoundError:
        print("❌ Validator data file not found")
        print("Make sure to run the test script first: ./test_zk_real_proofs.sh")
    except Exception as e:
        print(f"❌ Error: {e}")
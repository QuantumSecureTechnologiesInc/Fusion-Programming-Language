# Chapter 26: Blockchain Development in Fusion

Fusion's memory safety, linear types, and cryptographic primitives make it ideal for building secure blockchain applications. This chapter covers building blockchain infrastructure, smart contracts, DeFi protocols, and privacy-preserving systems.

## Core Concepts

### Blocks, Chains, Transactions

A blockchain is an immutable, append-only ledger of transactions grouped into blocks.

```fusion
// Block structure
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: Hash,
    nonce: u64,
    hash: Hash,
}

// Transaction structure
struct Transaction {
    sender: Address,
    recipient: Address,
    amount: u256,
    fee: u256,
    data: Vec<u8>,
    signature: Signature,
}

// Blockchain as a linked list of blocks
struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: u32,
    reward: u256,
}
```

### Cryptographic Primitives

```fusion
// SHA-256 hashing
use std::crypto::sha256;

let hash = sha256(b"hello world");
println(hash.hex()); // "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"

// Merkle tree for transaction verification
struct MerkleTree {
    leaves: Vec<Hash>,
    nodes: Vec<Hash>,
    root: Hash,
}

impl MerkleTree {
    fn new(transactions: &[Transaction]) -> Self {
        let leaves: Vec<Hash> = transactions.iter()
            .map(|tx| sha256(&tx.serialize()))
            .collect();
        
        let root = Self::build_tree(&leaves);
        Self { leaves, nodes: vec![], root }
    }
    
    fn build_tree(leaves: &[Hash]) -> Hash {
        if leaves.len() == 1 {
            return leaves[0];
        }
        
        let mut next_level = vec![];
        for pair in leaves.chunks(2) {
            let combined = if pair.len() == 2 {
                sha256(&[pair[0].as_bytes(), pair[1].as_bytes()].concat())
            } else {
                sha256(&[pair[0].as_bytes(), pair[0].as_bytes()].concat())
            };
            next_level.push(combined);
        }
        
        Self::build_tree(&next_level)
    }
    
    fn verify_proof(&self, tx_hash: Hash, proof: &[Hash], index: usize) -> bool {
        let mut current = tx_hash;
        let mut idx = index;
        
        for sibling in proof {
            current = if idx % 2 == 0 {
                sha256(&[current.as_bytes(), sibling.as_bytes()].concat())
            } else {
                sha256(&[sibling.as_bytes(), current.as_bytes()].concat())
            };
            idx /= 2;
        }
        
        current == self.root
    }
}
```

### Wallet Management

```fusion
// Ed25519 key pair for wallets
use std::crypto::ed25519;

struct Wallet {
    keypair: ed25519::KeyPair,
    address: Address,
}

impl Wallet {
    fn new() -> Self {
        let keypair = ed25519::KeyPair::generate();
        let address = Address::from_public_key(&keypair.public_key());
        Self { keypair, address }
    }
    
    fn sign(&self, transaction: &Transaction) -> Signature {
        self.keypair.sign(&transaction.serialize())
    }
    
    fn verify(public_key: &ed25519::PublicKey, message: &[u8], signature: &Signature) -> bool {
        ed25519::verify(public_key, message, signature)
    }
}

// HD Wallet (Hierarchical Deterministic)
struct HDWallet {
    master_key: ed25519::KeyPair,
    chain_code: [u8; 32],
}

impl HDWallet {
    fn from_seed(seed: &[u8]) -> Self {
        let master_key = ed25519::KeyPair::from_seed(seed);
        let chain_code = sha256(seed).as_bytes();
        Self { master_key, chain_code }
    }
    
    fn derive_child(&self, index: u32) -> HDWallet {
        let child_seed = sha256(&[
            self.master_key.secret_key().as_bytes(),
            &index.to_le_bytes(),
            &self.chain_code,
        ].concat());
        
        let child_key = ed25519::KeyPair::from_seed(&child_seed);
        let child_chain = sha256(&child_seed.as_bytes()).as_bytes();
        
        Self {
            master_key: child_key,
            chain_code: child_chain,
        }
    }
}
```

## Consensus Mechanisms

### Proof of Work (PoW)

```fusion
// Simple PoW implementation
fn mine_block(block: &mut Block, difficulty: u32) {
    let target = "0".repeat(difficulty as usize);
    
    loop {
        block.nonce += 1;
        block.hash = block.calculate_hash();
        
        if block.hash.hex().starts_with(&target) {
            break;
        }
    }
}

// Hash rate limiter
fn adjust_difficulty(chain: &Blockchain) -> u32 {
    let last_block = chain.chain.last().unwrap();
    let prev_block = chain.chain.get(chain.chain.len() - 2);
    
    match prev_block {
        Some(prev) => {
            let time_diff = last_block.timestamp - prev.timestamp;
            if time_diff < 10 {
                chain.difficulty + 1
            } else if time_diff > 30 && chain.difficulty > 1 {
                chain.difficulty - 1
            } else {
                chain.difficulty
            }
        }
        None => chain.difficulty,
    }
}
```

### Proof of Stake (PoS)

```fusion
// Stake-weighted validator selection
struct Validator {
    address: Address,
    stake: u256,
    reputation: u64,
    active: bool,
}

struct PoSConsensus {
    validators: Vec<Validator>,
    minimum_stake: u256,
    epoch_length: u64,
}

impl PoSConsensus {
    fn select_validator(&self, seed: &Hash) -> Option<&Validator> {
        let total_stake: u256 = self.validators.iter()
            .filter(|v| v.active && v.stake >= self.minimum_stake)
            .map(|v| v.stake)
            .sum();
        
        if total_stake == 0 {
            return None;
        }
        
        let mut target = u256::from_bytes(seed.as_bytes()) % total_stake;
        
        for validator in &self.validators {
            if validator.active && validator.stake >= self.minimum_stake {
                if target < validator.stake {
                    return Some(validator);
                }
                target -= validator.stake;
            }
        }
        
        None
    }
    
    fn validate_block(&self, block: &Block, validator: &Address) -> bool {
        // Check validator is in set
        self.validators.iter()
            .any(|v| &v.address == validator && v.active)
    }
}
```

### Delegated Proof of Stake (DPoS)

```fusion
// DPoS with elected delegates
struct DPoSConsensus {
    delegates: Vec<Delegate>,
    votes: HashMap<Address, Vec<Address>>,
    delegate_count: usize,
}

struct Delegate {
    address: Address,
    vote_count: u256,
    blocks_produced: u64,
    missed_blocks: u64,
}

impl DPoSConsensus {
    fn elect_delegates(&mut self) -> Vec<Address> {
        let mut sorted: Vec<_> = self.delegates.iter().collect();
        sorted.sort_by(|a, b| b.vote_count.cmp(&a.vote_count));
        
        sorted.iter()
            .take(self.delegate_count)
            .map(|d| d.address.clone())
            .collect()
    }
    
    fn vote(&mut self, voter: &Address, delegate: &Address) -> Result<(), VoteError> {
        if !self.delegates.iter().any(|d| &d.address == delegate) {
            return Err(VoteError::InvalidDelegate);
        }
        
        self.votes.entry(voter.clone())
            .or_insert_with(Vec::new)
            .push(delegate.clone());
        
        if let Some(d) = self.delegates.iter_mut().find(|d| &d.address == delegate) {
            d.vote_count += 1;
        }
        
        Ok(())
    }
}
```

### PBFT (Practical Byzantine Fault Tolerance)

```fusion
// PBFT consensus for permissioned networks
struct PBFTConsensus {
    nodes: Vec<NodeId>,
    view: u64,
    sequence: u64,
    prepare_messages: HashMap<NodeId, Message>,
    commit_messages: HashMap<NodeId, Message>,
}

impl PBFTConsensus {
    fn f(&self) -> usize {
        (self.nodes.len() - 1) / 3
    }
    
    fn quorum(&self) -> usize {
        2 * self.f() + 1
    }
    
    fn handle_prepare(&mut self, from: NodeId, message: Message) -> PBFTAction {
        self.prepare_messages.insert(from, message);
        
        if self.prepare_messages.len() >= self.quorum() {
            PBFTAction::SendCommit
        } else {
            PBFTAction::Wait
        }
    }
    
    fn handle_commit(&mut self, from: NodeId, message: Message) -> PBFTAction {
        self.commit_messages.insert(from, message);
        
        if self.commit_messages.len() >= self.quorum() {
            self.sequence += 1;
            PBFTAction::Finalize
        } else {
            PBFTAction::Wait
        }
    }
}
```

### Configuration and Selection

```fusion
// consensus.toml
// [consensus]
// type = "PoS"  // PoW, PoS, DPoS, PBFT
// [consensus.poS]
// minimum_stake = 32
// epoch_length = 32
// slashing_enabled = true
// [consensus.pbft]
// view_change_timeout = 5000
// max_block_size = 1MB

enum ConsensusType {
    PoW { difficulty: u32 },
    PoS { minimum_stake: u256, epoch_length: u64 },
    DPoS { delegate_count: usize },
    PBFT { view_change_timeout: u64 },
}

impl ConsensusType {
    fn from_config(config: &ConsensusConfig) -> Self {
        match config.consensus_type.as_str() {
            "PoW" => ConsensusType::PoW {
                difficulty: config.difficulty.unwrap_or(4),
            },
            "PoS" => ConsensusType::PoS {
                minimum_stake: config.minimum_stake.unwrap_or(32),
                epoch_length: config.epoch_length.unwrap_or(32),
            },
            "DPoS" => ConsensusType::DPoS {
                delegate_count: config.delegate_count.unwrap_or(21),
            },
            "PBFT" => ConsensusType::PBFT {
                view_change_timeout: config.view_change_timeout.unwrap_or(5000),
            },
            _ => panic!("Unknown consensus type: {}", config.consensus_type),
        }
    }
}
```

## Smart Contracts

### Contract Deployment

```fusion
// Smart contract bytecode
struct SmartContract {
    address: Address,
    bytecode: Vec<u8>,
    storage: HashMap<U256, U256>,
    balance: U256,
    owner: Address,
}

// Simple contract VM
enum Opcode {
    Stop,
    Add,
    Sub,
    Mul,
    Div,
    SStore,
    SLoad,
    Push(U256),
    Pop,
    Dup,
    Swap,
    Jmp(usize),
    JmpI(usize),
    Call(usize),
    Return,
    Log(u8),
}

struct ContractVM {
    stack: Vec<U256>,
    memory: Vec<u8>,
    pc: usize,
    gas: u64,
}

impl ContractVM {
    fn execute(&mut self, bytecode: &[u8], storage: &mut HashMap<U256, U256>) -> Result<Vec<u8>, VMError> {
        loop {
            if self.pc >= bytecode.len() {
                return Err(VMError::InvalidProgramCounter);
            }
            
            let opcode = Opcode::decode(bytecode[self.pc]);
            self.pc += 1;
            self.gas -= 1;
            
            if self.gas == 0 {
                return Err(VMError::OutOfGas);
            }
            
            match opcode {
                Opcode::Stop => return Ok(vec![]),
                Opcode::Add => {
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(a + b);
                }
                Opcode::SStore => {
                    let key = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    storage.insert(key, value);
                }
                Opcode::SLoad => {
                    let key = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let value = storage.get(&key).copied().unwrap_or(U256::zero());
                    self.stack.push(value);
                }
                Opcode::Push(val) => self.stack.push(val),
                _ => {}
            }
        }
    }
}
```

### Contract Calls

```fusion
// Contract interaction
struct ContractCaller {
    from: Address,
    to: Address,
    value: U256,
    data: Vec<u8>,
    gas: u64,
}

impl ContractCaller {
    fn call(&self, blockchain: &mut Blockchain) -> Result<CallResult, CallError> {
        let contract = blockchain.get_contract(&self.to)
            .ok_or(CallError::ContractNotFound)?;
        
        let mut vm = ContractVM::new(self.gas);
        let result = vm.execute(&contract.bytecode, &mut contract.storage.clone())?;
        
        // Transfer value
        blockchain.transfer(&self.from, &self.to, self.value)?;
        
        Ok(CallResult {
            success: true,
            return_data: result,
            gas_used: self.gas - vm.gas,
        })
    }
    
    fn static_call(&self, blockchain: &Blockchain) -> Result<Vec<u8>, CallError> {
        let contract = blockchain.get_contract(&self.to)
            .ok_or(CallError::ContractNotFound)?;
        
        let mut vm = ContractVM::new(self.gas);
        vm.execute(&contract.bytecode, &mut contract.storage.clone())
    }
}
```

### State Management

```fusion
// Persistent contract state
struct ContractState {
    storage: HashMap<U256, U256>,
    transient: HashMap<U256, U256>,
    logs: Vec<LogEntry>,
}

struct LogEntry {
    topics: Vec<Hash>,
    data: Vec<u8>,
    block_number: u64,
}

impl ContractState {
    fn commit(&mut self) {
        // Transient storage is cleared after transaction
        self.transient.clear();
    }
    
    fn revert(&mut self, checkpoint: StateCheckpoint) {
        self.storage = checkpoint.storage;
        self.logs.truncate(checkpoint.log_index);
    }
    
    fn snapshot(&self) -> StateCheckpoint {
        StateCheckpoint {
            storage: self.storage.clone(),
            log_index: self.logs.len(),
        }
    }
}
```

### Contract Upgrades

```fusion
// Upgradeable contract pattern
struct ProxyContract {
    implementation: Address,
    admin: Address,
    initialized: bool,
}

impl ProxyContract {
    fn upgrade(&mut self, new_implementation: Address, caller: &Address) -> Result<(), UpgradeError> {
        if caller != &self.admin {
            return Err(UpgradeError::NotAdmin);
        }
        
        if !self.is_valid_implementation(&new_implementation) {
            return Err(UpgradeError::InvalidImplementation);
        }
        
        self.implementation = new_implementation;
        Ok(())
    }
    
    fn delegate_call(&self, data: &[u8], storage: &mut HashMap<U256, U256>) -> Result<Vec<u8>, VMError> {
        let implementation = self.get_implementation_code()?;
        let mut vm = ContractVM::new(100000);
        vm.execute(&implementation, storage)
    }
}
```

## Token Standards

### ERC-20 (Fungible Tokens)

```fusion
struct ERC20Token {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: U256,
    balances: HashMap<Address, U256>,
    allowances: HashMap<Address, HashMap<Address, U256>>,
}

impl ERC20Token {
    fn transfer(&mut self, from: &Address, to: &Address, amount: U256) -> Result<(), TokenError> {
        let balance = self.balances.get(from).unwrap_or(&U256::zero());
        if *balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        self.balances.insert(*from, balance - amount);
        self.balances.entry(*to).and_modify(|e| *e += amount).or_insert(amount);
        
        Ok(())
    }
    
    fn approve(&mut self, owner: &Address, spender: &Address, amount: U256) {
        self.allowances
            .entry(*owner)
            .or_insert_with(HashMap::new)
            .insert(*spender, amount);
    }
    
    fn transfer_from(&mut self, owner: &Address, from: &Address, to: &Address, amount: U256) -> Result<(), TokenError> {
        let allowance = self.allowances
            .get(owner)
            .and_then(|m| m.get(from))
            .unwrap_or(&U256::zero());
        
        if *allowance < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        
        self.transfer(from, to, amount)?;
        self.allowances.get_mut(owner).unwrap().insert(*from, allowance - amount);
        
        Ok(())
    }
}
```

### ERC-721 (NFTs)

```fusion
struct ERC721Token {
    name: String,
    symbol: String,
    owners: HashMap<u64, Address>,
    balances: HashMap<Address, u64>,
    token_approvals: HashMap<u64, Address>,
    operator_approvals: HashMap<Address, HashMap<Address, bool>>,
}

impl ERC721Token {
    fn mint(&mut self, to: &Address, token_id: u64) -> Result<(), TokenError> {
        if self.owners.contains_key(&token_id) {
            return Err(TokenError::AlreadyMinted);
        }
        
        self.owners.insert(token_id, *to);
        self.balances.entry(*to).and_modify(|e| *e += 1).or_insert(1);
        
        Ok(())
    }
    
    fn transfer_from(&mut self, from: &Address, to: &Address, token_id: u64) -> Result<(), TokenError> {
        let owner = self.owners.get(&token_id).ok_or(TokenError::NotFound)?;
        if owner != from {
            return Err(TokenError::NotOwner);
        }
        
        self.owners.insert(token_id, *to);
        self.balances.entry(*from).and_modify(|e| *e -= 1);
        self.balances.entry(*to).and_modify(|e| *e += 1).or_insert(1);
        
        Ok(())
    }
    
    fn approve(&mut self, to: &Address, token_id: u64) {
        self.token_approvals.insert(token_id, *to);
    }
}
```

### ERC-1155 (Multi-Token)

```fusion
struct ERC1155Token {
    balances: HashMap<u64, HashMap<Address, U256>>,
    operator_approvals: HashMap<Address, HashMap<Address, bool>>,
}

impl ERC1155Token {
    fn safe_batch_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        ids: &[u64],
        amounts: &[U256],
    ) -> Result<(), TokenError> {
        if ids.len() != amounts.len() {
            return Err(TokenError::LengthMismatch);
        }
        
        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let balance = self.balances
                .get(id)
                .and_then(|m| m.get(from))
                .unwrap_or(&U256::zero());
            
            if *balance < *amount {
                return Err(TokenError::InsufficientBalance);
            }
            
            self.balances.get_mut(id).unwrap().insert(*from, balance - amount);
            self.balances.entry(*id).or_insert_with(HashMap::new)
                .entry(*to).and_modify(|e| *e += amount).or_insert(*amount);
        }
        
        Ok(())
    }
}
```

### Custom Tokens

```fusion
// Governance token with voting power
struct GovernanceToken {
    base: ERC20Token,
    voting_power: HashMap<Address, U256>,
    delegation: HashMap<Address, Address>,
}

impl GovernanceToken {
    fn delegate(&mut self, delegator: &Address, delegate: &Address) {
        let power = self.base.balances.get(delegator).copied().unwrap_or(U256::zero());
        
        // Remove old delegation
        if let Some(old) = self.delegation.get(delegator) {
            self.voting_power.entry(*old).and_modify(|e| *e -= power);
        }
        
        // Add new delegation
        self.delegation.insert(*delegator, *delegate);
        self.voting_power.entry(*delegate).and_modify(|e| *e += power).or_insert(power);
    }
    
    fn get_votes(&self, account: &Address) -> U256 {
        self.voting_power.get(account).copied().unwrap_or(U256::zero())
    }
}
```

## DeFi

### Liquidity Pools (AMM)

```fusion
// Constant product AMM (x * y = k)
struct LiquidityPool {
    token_a: Address,
    token_b: Address,
    reserve_a: U256,
    reserve_b: U256,
    total_shares: U256,
    shares: HashMap<Address, U256>,
}

impl LiquidityPool {
    fn add_liquidity(&mut self, provider: &Address, amount_a: U256, amount_b: U256) -> U256 {
        let total_value = self.reserve_a * amount_b + self.reserve_b * amount_a;
        let shares = if self.total_shares == 0 {
            (amount_a * amount_b).sqrt()
        } else {
            total_value / (self.reserve_a + self.reserve_b)
        };
        
        self.reserve_a += amount_a;
        self.reserve_b += amount_b;
        self.total_shares += shares;
        self.shares.entry(*provider).and_modify(|e| *e += shares).or_insert(shares);
        
        shares
    }
    
    fn swap(&mut self, input_amount: U256, token_in: &Address) -> Result<U256, SwapError> {
        let (reserve_in, reserve_out) = if *token_in == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };
        
        let input_with_fee = input_amount * 997 / 1000;
        let output = (reserve_out * input_with_fee) / (reserve_in + input_with_fee);
        
        if *token_in == self.token_a {
            self.reserve_a += input_amount;
            self.reserve_b -= output;
        } else {
            self.reserve_b += input_amount;
            self.reserve_a -= output;
        }
        
        Ok(output)
    }
    
    fn get_price(&self, token_in: &Address) -> f64 {
        if *token_in == self.token_a {
            self.reserve_b as f64 / self.reserve_a as f64
        } else {
            self.reserve_a as f64 / self.reserve_b as f64
        }
    }
}
```

### Lending/Borrowing

```fusion
// Simple lending protocol
struct LendingPool {
    deposits: HashMap<Address, U256>,
    borrows: HashMap<Address, U256>,
    interest_rate: f64,
    collateral_ratio: f64,
}

impl LendingPool {
    fn deposit(&mut self, lender: &Address, amount: U256) {
        self.deposits.entry(*lender).and_modify(|e| *e += amount).or_insert(amount);
    }
    
    fn borrow(&mut self, borrower: &Address, amount: U256, collateral: U256) -> Result<(), LendingError> {
        let required_collateral = amount as f64 * self.collateral_ratio;
        if (collateral as f64) < required_collateral {
            return Err(LendingError::InsufficientCollateral);
        }
        
        let total_deposits: U256 = self.deposits.values().sum();
        if amount > total_deposits {
            return Err(LendingError::InsufficientLiquidity);
        }
        
        self.borrows.entry(*borrower).and_modify(|e| *e += amount).or_insert(amount);
        Ok(())
    }
    
    fn accrue_interest(&mut self) {
        for borrow in self.borrows.values_mut() {
            *borrow = (*borrow as f64 * (1.0 + self.interest_rate)) as U256;
        }
    }
}
```

### DEX Functionality

```fusion
// Order book DEX
struct OrderBook {
    bids: BTreeMap<U256, Vec<Order>>,
    asks: BTreeMap<U256, Vec<Order>>,
}

struct Order {
    id: Hash,
    user: Address,
    side: Side,
    price: U256,
    amount: U256,
    filled: U256,
}

enum Side {
    Buy,
    Sell,
}

impl OrderBook {
    fn place_order(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = vec![];
        
        match order.side {
            Side::Buy => {
                while let Some((&price, asks)) = self.asks.iter_mut().next() {
                    if price > order.price || order.amount == order.filled {
                        break;
                    }
                    
                    for ask in asks.iter_mut() {
                        let fill_amount = (order.amount - order.filled).min(ask.amount - ask.filled);
                        
                        trades.push(Trade {
                            buyer: order.user,
                            seller: ask.user,
                            price,
                            amount: fill_amount,
                        });
                        
                        ask.filled += fill_amount;
                        // Update order...
                    }
                    
                    self.asks.retain(|_, orders| orders.iter().any(|o| o.filled < o.amount));
                }
            }
            Side::Sell => {
                // Similar logic for sells
            }
        }
        
        trades
    }
}
```

## Privacy

### Stealth Addresses

```fusion
// One-time stealth addresses
struct StealthAddress {
    view_key: PublicKey,
    spend_key: PublicKey,
}

impl StealthAddress {
    fn generate_stealth(pub_key: &PublicKey, random: &[u8; 32]) -> (PublicKey, [u8; 32]) {
        let ephemeral = PublicKey::from_random(random);
        let shared_secret = pub_key.diffie_hellman(&ephemeral);
        
        let stealth = pub_key.add_point(&shared_secret.hash_to_point());
        (stealth, ephemeral.to_bytes())
    }
    
    fn scan_for_transactions(
        view_key: &SecretKey,
        spend_key: &SecretKey,
        blockchain: &Blockchain,
    ) -> Vec<Transaction> {
        blockchain.transactions.iter()
            .filter(|tx| {
                let ephemeral = PublicKey::from_bytes(&tx.data[0..32]);
                let shared_secret = view_key.diffie_hellman(&ephemeral);
                let expected = spend_key.public_key().add_point(&shared_secret.hash_to_point());
                
                tx.recipient == expected.to_address()
            })
            .cloned()
            .collect()
    }
}
```

### Confidential Transactions

```fusion
// Pedersen commitments for amount hiding
struct ConfidentialAmount {
    commitment: PedersenCommitment,
    proof: RangeProof,
}

impl ConfidentialAmount {
    fn create(amount: U256, blinding: &Scalar) -> Self {
        let commitment = PedersenCommitment::commit(amount, blinding);
        let proof = RangeProof::prove(amount, blinding, 64);
        
        Self { commitment, proof }
    }
    
    fn verify(&self) -> bool {
        self.proof.verify(&self.commitment)
    }
    
    fn add(a: &ConfidentialAmount, b: &ConfidentialAmount) -> Self {
        Self {
            commitment: a.commitment.add(&b.commitment),
            proof: RangeProof::prove_homomorphic(&a.proof, &b.proof),
        }
    }
}
```

### Shielded Pools

```fusion
// Zcash-style shielded pool
struct ShieldedPool {
    commitments: Vec<PedersenCommitment>,
    nullifiers: Vec<Hash>,
    merkle_tree: IncrementalMerkleTree,
}

impl ShieldedPool {
    fn shield(&mut self, input: ClearAmount, note: ShieldedNote) -> ShieldedProof {
        // Create commitment
        let commitment = note.commit();
        self.merkle_tree.append(commitment);
        
        // Generate proof
        let proof = ShieldedProof::create(
            &note,
            &self.merkle_tree.root(),
            &self.nullifiers,
        );
        
        self.commitments.push(commitment);
        proof
    }
    
    fn unshield(&mut self, proof: ShieldedProof, output: ClearAmount) -> Result<(), ShieldError> {
        if !proof.verify(&self.merkle_tree.root()) {
            return Err(ShieldError::InvalidProof);
        }
        
        let nullifier = proof.nullifier();
        if self.nullifiers.contains(&nullifier) {
            return Err(ShieldError::DoubleSpend);
        }
        
        self.nullifiers.push(nullifier);
        Ok(())
    }
}
```

### Zero-Knowledge Proofs

```fusion
// zk-SNARK proof system
struct ZKProof {
    proof: Groth16Proof,
    public_inputs: Vec<FieldElement>,
}

impl ZKProof {
    fn prove(circuit: &Circuit, witness: &[FieldElement]) -> Self {
        let proof = Groth16::prove(circuit, witness);
        let public_inputs = circuit.public_inputs(witness);
        
        Self { proof, public_inputs }
    }
    
    fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        Groth16::verify(verifying_key, &self.public_inputs, &self.proof)
    }
}

// Example: Proving knowledge of a hash preimage
fn prove_knowledge_of_preimage(preimage: &[u8], hash: &Hash) -> ZKProof {
    let circuit = Circuit::new(|| {
        let x = witness();
        let h = sha256(x);
        constrain!(h == hash);
    });
    
    let witness = vec![FieldElement::from_bytes(preimage)];
    ZKProof::prove(&circuit, &witness)
}
```

## Governance

### On-Chain Proposals

```fusion
struct Proposal {
    id: u64,
    proposer: Address,
    title: String,
    description: String,
    calls: Vec<ProposalCall>,
    start_block: u64,
    end_block: u64,
    for_votes: U256,
    against_votes: U256,
    executed: bool,
}

struct ProposalCall {
    target: Address,
    value: U256,
    data: Vec<u8>,
}

impl Proposal {
    fn create(
        proposer: Address,
        title: String,
        description: String,
        calls: Vec<ProposalCall>,
        current_block: u64,
        voting_period: u64,
    ) -> Self {
        Self {
            id: 0, // Assigned by governor
            proposer,
            title,
            description,
            calls,
            start_block: current_block + 1,
            end_block: current_block + voting_period,
            for_votes: U256::zero(),
            against_votes: U256::zero(),
            executed: false,
        }
    }
    
    fn is_active(&self, current_block: u64) -> bool {
        current_block >= self.start_block && current_block <= self.end_block
    }
    
    fn is_passed(&self, quorum: U256) -> bool {
        self.for_votes > self.against_votes && self.for_votes + self.against_votes >= quorum
    }
}
```

### Voting Mechanisms

```fusion
// Multiple voting strategies
enum VotingStrategy {
    TokenWeighted,
    Quadratic,
    Conviction,
    Holographic,
}

struct VotingSystem {
    strategy: VotingStrategy,
    proposals: Vec<Proposal>,
    votes: HashMap<Address, HashMap<u64, Vote>>,
}

struct Vote {
    voter: Address,
    proposal_id: u64,
    support: bool,
    weight: U256,
    block: u64,
}

impl VotingSystem {
    fn vote(&mut self, voter: &Address, proposal_id: u64, support: bool, token_balance: U256) -> Result<(), VoteError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(VoteError::ProposalNotFound)?;
        
        if !proposal.is_active(current_block) {
            return Err(VoteError::VotingEnded);
        }
        
        let weight = match self.strategy {
            VotingStrategy::TokenWeighted => token_balance,
            VotingStrategy::Quadratic => token_balance.sqrt(),
            VotingStrategy::Conviction => token_balance * (current_block - self.votes.get(voter).map(|v| v.block).unwrap_or(0)),
            VotingStrategy::Holographic => token_balance,
        };
        
        if support {
            proposal.for_votes += weight;
        } else {
            proposal.against_votes += weight;
        }
        
        Ok(())
    }
}
```

### DAOs

```fusion
// DAO structure
struct DAO {
    governance: GovernanceToken,
    treasury: U256,
    members: Vec<Address>,
    proposals: Vec<Proposal>,
    quorum: U256,
    voting_period: u64,
    timelock_delay: u64,
}

impl DAO {
    fn propose(&mut self, proposer: &Address, proposal: Proposal) -> Result<u64, ProposalError> {
        let votes = self.governance.get_votes(proposer);
        if votes < self.proposal_threshold {
            return Err(ProposalError::InsufficientVotes);
        }
        
        let id = self.proposals.len() as u64;
        self.proposals.push(proposal);
        
        Ok(id)
    }
    
    fn execute(&mut self, proposal_id: u64, caller: &Address) -> Result<(), ExecutionError> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(ExecutionError::ProposalNotFound)?;
        
        if proposal.executed {
            return Err(ExecutionError::AlreadyExecuted);
        }
        
        if !proposal.is_passed(self.quorum) {
            return Err(ExecutionError::NotPassed);
        }
        
        // Execute calls through timelock
        for call in &proposal.calls {
            execute_call(call)?;
        }
        
        proposal.executed = true;
        Ok(())
    }
    
    fn add_member(&mut self, member: &Address) -> Result<(), DAOError> {
        if self.members.contains(member) {
            return Err(DAOError::AlreadyMember);
        }
        
        self.members.push(*member);
        Ok(())
    }
}
```

### Staking and Rewards

```fusion
struct StakingPool {
    stakers: HashMap<Address, StakeInfo>,
    reward_rate: U256,
    total_staked: U256,
    last_reward_block: u64,
}

struct StakeInfo {
    amount: U256,
    reward_debt: U256,
    lock_until: u64,
}

impl StakingPool {
    fn stake(&mut self, staker: &Address, amount: U256, lock_period: u64) {
        self.update_rewards(staker);
        
        self.stakers.entry(*staker).and_modify(|info| {
            info.amount += amount;
        }).insert(StakeInfo {
            amount,
            reward_debt: U256::zero(),
            lock_until: current_block + lock_period,
        });
        
        self.total_staked += amount;
    }
    
    fn claim_rewards(&mut self, staker: &Address) -> U256 {
        self.update_rewards(staker);
        
        let info = self.stakers.get_mut(staker).unwrap();
        let rewards = info.reward_debt;
        info.reward_debt = U256::zero();
        
        rewards
    }
    
    fn update_rewards(&mut self, staker: &Address) {
        let blocks_passed = current_block - self.last_reward_block;
        let total_reward = self.reward_rate * blocks_passed;
        
        if let Some(info) = self.stakers.get(staker) {
            let share = (info.amount * total_reward) / self.total_staked;
            info.reward_debt += share;
        }
        
        self.last_reward_block = current_block;
    }
}
```

## Networking

### P2P Networking

```fusion
// Peer-to-peer network layer
struct P2PNetwork {
    node_id: NodeId,
    peers: HashMap<NodeId, Peer>,
    listening_port: u16,
    max_peers: usize,
}

struct Peer {
    id: NodeId,
    address: SocketAddr,
    connection: TcpStream,
    last_seen: Instant,
    chain_height: u64,
}

impl P2PNetwork {
    fn connect_to_peer(&mut self, address: SocketAddr) -> Result<NodeId, NetworkError> {
        let stream = TcpStream::connect(address)?;
        let peer_id = NodeId::random();
        
        // Handshake
        let handshake = Handshake {
            version: PROTOCOL_VERSION,
            node_id: self.node_id,
            chain_height: self.get_chain_height(),
        };
        
        write_message(&stream, &handshake)?;
        let response: Handshake = read_message(&stream)?;
        
        self.peers.insert(peer_id, Peer {
            id: peer_id,
            address,
            connection: stream,
            last_seen: Instant::now(),
            chain_height: response.chain_height,
        });
        
        Ok(peer_id)
    }
    
    fn broadcast(&self, message: &NetworkMessage) {
        for peer in self.peers.values() {
            let _ = write_message(&peer.connection, message);
        }
    }
}
```

### Peer Discovery

```fusion
// Kademlia DHT for peer discovery
struct DHT {
    local_id: NodeId,
    k_buckets: Vec<KBucket>,
    alpha: usize,
    k: usize,
}

struct KBucket {
    nodes: VecDeque<NodeEntry>,
    last_updated: Instant,
}

struct NodeEntry {
    id: NodeId,
    address: SocketAddr,
    last_seen: Instant,
}

impl DHT {
    fn find_node(&self, target: &NodeId) -> Vec<NodeEntry> {
        let closest = self.closest_nodes(target, self.k);
        let mut queried = HashSet::new();
        let mut candidates = BinaryHeap::new();
        
        for node in &closest {
            candidates.push(DistanceKey::new(node, target));
        }
        
        loop {
            let next = candidates.pop().ok_or(DHTError::NoNodes)?;
            
            if queried.contains(&next.node.id) {
                continue;
            }
            
            queried.insert(next.node.id);
            let response = self.query_node(&next.node, target)?;
            
            for node in response {
                if !queried.contains(&node.id) {
                    candidates.push(DistanceKey::new(&node, target));
                }
            }
            
            if candidates.peek().map(|c| c.distance >= next.distance).unwrap_or(true) {
                return closest;
            }
        }
    }
    
    fn store(&mut self, key: Hash, value: Vec<u8>) {
        let closest = self.closest_nodes(&key.to_node_id(), self.k);
        for node in closest {
            let _ = self.store_value(&node, key, value.clone());
        }
    }
}
```

### Chain Synchronization

```fusion
// Fast block sync protocol
struct ChainSync {
    peer_heights: HashMap<NodeId, u64>,
    our_height: u64,
    sync_state: SyncState,
}

enum SyncState {
    Idle,
    HeaderSync,
    BlockSync,
    StateSync,
}

impl ChainSync {
    fn start_sync(&mut self, network: &P2PNetwork) -> Result<(), SyncError> {
        // Get best peer
        let best_peer = self.peer_heights.iter()
            .max_by_key(|(_, &h)| h)
            .map(|(id, _)| *id)
            .ok_or(SyncError::NoPeers)?;
        
        let target_height = self.peer_heights[&best_peer];
        
        // Sync headers
        self.sync_state = SyncState::HeaderSync;
        let headers = network.request_headers(&best_peer, self.our_height, 2000)?;
        
        for header in headers {
            if !header.verify() {
                return Err(SyncError::InvalidHeader);
            }
            self.our_height += 1;
        }
        
        // Sync blocks
        self.sync_state = SyncState::BlockSync;
        while self.our_height < target_height {
            let blocks = network.request_blocks(&best_peer, self.our_height + 1, 100)?;
            
            for block in blocks {
                self.process_block(block)?;
                self.our_height += 1;
            }
        }
        
        self.sync_state = SyncState::Idle;
        Ok(())
    }
    
    fn process_block(&mut self, block: Block) -> Result<(), SyncError> {
        if !block.verify() {
            return Err(SyncError::InvalidBlock);
        }
        
        // Add to chain
        Ok(())
    }
}
```

### Gossip Protocol

```fusion
// Gossip for transaction propagation
struct GossipProtocol {
    seen: BloomFilter,
    pending: Vec<Transaction>,
    fanout: usize,
}

impl GossipProtocol {
    fn broadcast_transaction(&mut self, tx: &Transaction, network: &P2PNetwork) {
        if self.seen.contains(&tx.hash()) {
            return;
        }
        
        self.seen.add(&tx.hash());
        self.pending.push(tx.clone());
        
        // Select random peers
        let peers = network.random_peers(self.fanout);
        
        for peer in peers {
            let _ = network.send_transaction(&peer, tx);
        }
    }
    
    fn handle_transaction(&mut self, tx: Transaction, from: &NodeId, network: &P2PNetwork) {
        if self.seen.contains(&tx.hash()) {
            return;
        }
        
        self.seen.add(&tx.hash());
        self.pending.push(tx.clone());
        
        // Forward to other peers (not the one we received from)
        let peers = network.random_peers_excluding(self.fanout, from);
        
        for peer in peers {
            let _ = network.send_transaction(&peer, &tx);
        }
    }
}
```

## Storage

### Block Storage

```fusion
// Persistent block storage
struct BlockStorage {
    db: Database,
    block_index: HashMap<Hash, u64>,
    height_index: HashMap<u64, Hash>,
}

impl BlockStorage {
    fn new(path: &str) -> Self {
        let db = Database::open(path).expect("Failed to open database");
        Self {
            db,
            block_index: HashMap::new(),
            height_index: HashMap::new(),
        }
    }
    
    fn store_block(&mut self, block: &Block) -> Result<(), StorageError> {
        let key = block.hash.as_bytes();
        let value = block.serialize();
        
        self.db.put(key, &value)?;
        self.block_index.insert(block.hash, block.index);
        self.height_index.insert(block.index, block.hash);
        
        Ok(())
    }
    
    fn get_block(&self, hash: &Hash) -> Result<Option<Block>, StorageError> {
        match self.db.get(hash.as_bytes())? {
            Some(data) => Ok(Some(Block::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        match self.height_index.get(&height) {
            Some(hash) => self.get_block(hash),
            None => Ok(None),
        }
    }
}
```

### State Storage

```fusion
// Trie-based state storage
struct StateStorage {
    trie: Merkle PatriciaTrie,
    db: Database,
}

impl StateStorage {
    fn get_account(&self, address: &Address) -> Result<Option<Account>, StorageError> {
        let key = sha256(address.as_bytes());
        match self.trie.get(&key)? {
            Some(data) => Ok(Some(Account::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    fn set_account(&mut self, address: &Address, account: &Account) -> Result<(), StorageError> {
        let key = sha256(address.as_bytes());
        let value = account.serialize();
        
        self.trie.insert(key, value);
        self.db.put(self.trie.root().as_bytes(), &self.trie.serialize())?;
        
        Ok(())
    }
    
    fn get_storage(&self, address: &Address, slot: &U256) -> Result<U256, StorageError> {
        let account = self.get_account(address)?
            .ok_or(StorageError::AccountNotFound)?;
        
        Ok(account.storage.get(slot).copied().unwrap_or(U256::zero()))
    }
    
    fn set_storage(&mut self, address: &Address, slot: U256, value: U256) -> Result<(), StorageError> {
        let mut account = self.get_account(address)?
            .ok_or(StorageError::AccountNotFound)?;
        
        account.storage.insert(slot, value);
        self.set_account(address, &account)
    }
}
```

### Snapshots and Revert

```fusion
// State snapshots for fast revert
struct SnapshotManager {
    snapshots: Vec<Snapshot>,
    max_snapshots: usize,
}

struct Snapshot {
    id: u64,
    state_root: Hash,
    block_number: u64,
    timestamp: u64,
}

impl SnapshotManager {
    fn create_snapshot(&mut self, state_root: Hash, block_number: u64) -> u64 {
        let id = self.snapshots.len() as u64;
        
        self.snapshots.push(Snapshot {
            id,
            state_root,
            block_number,
            timestamp: current_time(),
        });
        
        // Keep only recent snapshots
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
        
        id
    }
    
    fn revert_to_snapshot(&self, snapshot_id: u64, storage: &mut StateStorage) -> Result<(), RevertError> {
        let snapshot = self.snapshots.iter()
            .find(|s| s.id == snapshot_id)
            .ok_or(RevertError::SnapshotNotFound)?;
        
        storage.restore_state(&snapshot.state_root)?;
        
        Ok(())
    }
    
    fn prune_old_snapshots(&mut self, keep: usize) {
        if self.snapshots.len() > keep {
            self.snapshots.drain(0..self.snapshots.len() - keep);
        }
    }
}
```

## Cross-Chain

### Bridges

```fusion
// Cross-chain bridge
struct Bridge {
    source_chain: ChainId,
    target_chain: ChainId,
    validator_set: Vec<Address>,
    nonce: u64,
    locked_assets: HashMap<Hash, LockedAsset>,
}

struct LockedAsset {
    token: Address,
    amount: U256,
    sender: Address,
    recipient: Address,
    source_tx: Hash,
}

impl Bridge {
    fn lock_tokens(&mut self, sender: &Address, token: &Address, amount: U256, recipient: Address) -> Result<Hash, BridgeError> {
        // Lock tokens in bridge contract
        let lock_tx = self.create_lock_transaction(token, amount)?;
        
        let asset = LockedAsset {
            token: *token,
            amount,
            sender: *sender,
            recipient,
            source_tx: lock_tx,
        };
        
        let asset_hash = sha256(&asset.serialize());
        self.locked_assets.insert(asset_hash, asset);
        
        // Emit event for validators
        self.emit_lock_event(asset_hash);
        
        Ok(asset_hash)
    }
    
    fn release_tokens(&mut self, asset_hash: &Hash, signatures: &[Signature]) -> Result<(), BridgeError> {
        let asset = self.locked_assets.get(asset_hash)
            .ok_or(BridgeError::AssetNotFound)?;
        
        // Verify signatures
        if !self.verify_signatures(asset_hash, signatures) {
            return Err(BridgeError::InvalidSignatures);
        }
        
        // Release on target chain
        self.release_on_target(asset)?;
        
        // Remove from locked assets
        self.locked_assets.remove(asset_hash);
        
        Ok(())
    }
    
    fn verify_signatures(&self, asset_hash: &Hash, signatures: &[Signature]) -> bool {
        let required = (self.validator_set.len() * 2) / 3 + 1;
        
        let valid = signatures.iter()
            .filter(|sig| {
                self.validator_set.iter().any(|v| v.verify(asset_hash.as_bytes(), sig))
            })
            .count();
        
        valid >= required
    }
}
```

### Cross-Chain Messaging

```fusion
// Cross-chain message protocol
struct CrossChainMessage {
    source: ChainId,
    destination: ChainId,
    nonce: u64,
    sender: Address,
    payload: Vec<u8>,
    signature: Signature,
}

struct MessageRelay {
    messages: Vec<CrossChainMessage>,
    processed: HashSet<u64>,
}

impl MessageRelay {
    fn relay_message(&mut self, message: CrossChainMessage, target_chain: &mut Chain) -> Result<(), RelayError> {
        // Verify signature
        if !message.verify_signature() {
            return Err(RelayError::InvalidSignature);
        }
        
        // Check nonce
        if self.processed.contains(&message.nonce) {
            return Err(RelayError::DuplicateMessage);
        }
        
        // Deliver to target chain
        target_chain.process_cross_chain_message(&message)?;
        
        self.processed.insert(message.nonce);
        self.messages.push(message);
        
        Ok(())
    }
}
```

### Layer 2 Scaling

```fusion
// State channels for off-chain scaling
struct StateChannel {
    participants: [Address; 2],
    balances: [U256; 2],
    nonce: u64,
    state: ChannelState,
    settlement_delay: u64,
}

enum ChannelState {
    Opening,
    Open,
    Closing,
    Closed,
}

impl StateChannel {
    fn update_state(&mut self, new_balances: [U256; 2], signatures: &[Signature; 2]) -> Result<(), ChannelError> {
        // Verify both parties signed
        for (i, sig) in signatures.iter().enumerate() {
            if !self.participants[i].verify(&new_balances.serialize(), sig) {
                return Err(ChannelError::InvalidSignature);
            }
        }
        
        self.balances = new_balances;
        self.nonce += 1;
        
        Ok(())
    }
    
    fn close(&mut self, final_state: [U256; 2], signatures: &[Signature; 2]) -> Result<(), ChannelError> {
        self.update_state(final_state, signatures)?;
        self.state = ChannelState::Closing;
        
        // Start settlement delay
        Ok(())
    }
}

// Rollup for batch processing
struct Rollup {
    batches: Vec<Batch>,
    state_root: Hash,
    fraud_proofs: Vec<FraudProof>,
}

struct Batch {
    transactions: Vec<Transaction>,
    state_root: Hash,
    signature: Signature,
}

impl Rollup {
    fn submit_batch(&mut self, batch: Batch) -> Result<u64, RollupError> {
        // Verify batch signature
        if !batch.verify_signature() {
            return Err(RollupError::InvalidBatch);
        }
        
        let index = self.batches.len() as u64;
        self.batches.push(batch);
        self.state_root = batch.state_root;
        
        Ok(index)
    }
    
    fn submit_fraud_proof(&mut self, proof: FraudProof) -> Result<(), RollupError> {
        if !proof.verify() {
            return Err(RollupError::InvalidProof);
        }
        
        // Revert to previous valid state
        self.revert_batch(proof.batch_index)?;
        
        self.fraud_proofs.push(proof);
        
        Ok(())
    }
    
    fn revert_batch(&mut self, batch_index: u64) -> Result<(), RollupError> {
        // Remove invalid batch and all subsequent batches
        self.batches.truncate(batch_index as usize);
        
        // Recompute state root
        if let Some(last) = self.batches.last() {
            self.state_root = last.state_root;
        } else {
            self.state_root = Hash::zero();
        }
        
        Ok(())
    }
}
```

## Complete Examples

### Create a Simple Blockchain

```fusion
// Simple blockchain implementation
struct SimpleBlockchain {
    chain: Vec<Block>,
    difficulty: u32,
    reward: U256,
}

impl SimpleBlockchain {
    fn new(difficulty: u32, reward: U256) -> Self {
        let genesis = Block::genesis();
        Self {
            chain: vec![genesis],
            difficulty,
            reward,
        }
    }
    
    fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_hash = self.chain.last().unwrap().hash;
        
        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: current_time(),
            transactions,
            previous_hash,
            nonce: 0,
            hash: Hash::zero(),
        };
        
        mine_block(&mut block, self.difficulty);
        self.chain.push(block);
    }
    
    fn validate_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            
            if current.hash != current.calculate_hash() {
                return false;
            }
            
            if current.previous_hash != previous.hash {
                return false;
            }
            
            if !current.verify_transactions() {
                return false;
            }
        }
        
        true
    }
    
    fn get_balance(&self, address: &Address) -> U256 {
        let mut balance = U256::zero();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.sender == *address {
                    balance -= tx.amount + tx.fee;
                }
                if tx.recipient == *address {
                    balance += tx.amount;
                }
            }
        }
        
        balance
    }
}

// Usage
let mut blockchain = SimpleBlockchain::new(4, U256::from(50));

// Create transactions
let wallet1 = Wallet::new();
let wallet2 = Wallet::new();

let tx1 = Transaction::new(&wallet1, wallet2.address, U256::from(10), U256::from(1));
wallet1.sign(&tx1);

blockchain.add_block(vec![tx1]);
```

### Deploy an ERC-20 Token

```fusion
// Deploy a new ERC-20 token
fn deploy_token(
    name: String,
    symbol: String,
    decimals: u8,
    initial_supply: U256,
    deployer: &Address,
) -> Contract {
    let bytecode = compile_token_contract();
    let mut storage = HashMap::new();
    
    // Initialize storage slots
    storage.insert(keccak256(b"name"), U256::from_bytes(&name.into_bytes()));
    storage.insert(keccak256(b"symbol"), U256::from_bytes(&symbol.into_bytes()));
    storage.insert(keccak256(b"decimals"), U256::from(decimals as u64));
    storage.insert(keccak256(b"totalSupply"), initial_supply);
    storage.insert(keccak256(&balance_key(deployer)), initial_supply);
    
    Contract {
        address: Address::from_bytecode(&bytecode),
        bytecode,
        storage,
        balance: U256::zero(),
        owner: *deployer,
    }
}

// Interact with deployed token
fn transfer_tokens(
    token: &mut Contract,
    from: &Address,
    to: &Address,
    amount: U256,
) -> Result<(), TokenError> {
    let balance_slot = balance_key(from);
    let balance = token.storage.get(&balance_slot).copied().unwrap_or(U256::zero());
    
    if balance < amount {
        return Err(TokenError::InsufficientBalance);
    }
    
    token.storage.insert(balance_slot, balance - amount);
    token.storage.entry(balance_key(to)).and_modify(|e| *e += amount).or_insert(amount);
    
    Ok(())
}
```

### Build a DAO

```fusion
// Create a DAO with governance
fn create_dao(
    name: String,
    governance_token: Contract,
    initial_members: Vec<Address>,
) -> DAO {
    let mut dao = DAO {
        governance: GovernanceToken::new(governance_token),
        treasury: U256::zero(),
        members: initial_members.clone(),
        proposals: vec![],
        quorum: U256::from(1000),
        voting_period: 50400, // ~1 week
        timelock_delay: 172800, // ~2 days
    };
    
    // Mint governance tokens to initial members
    let tokens_per_member = U256::from(10000);
    for member in &initial_members {
        dao.governance.mint(member, tokens_per_member);
    }
    
    dao
}

// Create a proposal
fn create_proposal(
    dao: &mut DAO,
    proposer: &Address,
    title: String,
    description: String,
    calls: Vec<ProposalCall>,
) -> Result<u64, ProposalError> {
    let votes = dao.governance.get_votes(proposer);
    if votes < U256::from(100) {
        return Err(ProposalError::InsufficientVotes);
    }
    
    let proposal = Proposal::create(
        *proposer,
        title,
        description,
        calls,
        current_block(),
        dao.voting_period,
    );
    
    let id = dao.proposals.len() as u64;
    dao.proposals.push(proposal);
    
    Ok(id)
}
```

### Create an AMM Liquidity Pool

```fusion
// Deploy a new AMM liquidity pool
fn deploy_amm(
    token_a: Address,
    token_b: Address,
    fee_rate: u64,
) -> LiquidityPool {
    LiquidityPool {
        token_a,
        token_b,
        reserve_a: U256::zero(),
        reserve_b: U256::zero(),
        total_shares: U256::zero(),
        shares: HashMap::new(),
        fee_rate,
    }
}

// Add liquidity and get LP tokens
fn add_liquidity(
    pool: &mut LiquidityPool,
    provider: &Address,
    amount_a: U256,
    amount_b: U256,
) -> U256 {
    // Calculate optimal amounts
    let (optimal_a, optimal_b) = if pool.reserve_a == U256::zero() && pool.reserve_b == U256::zero() {
        (amount_a, amount_b)
    } else {
        let optimal_b = amount_a * pool.reserve_b / pool.reserve_a;
        if optimal_b <= amount_b {
            (amount_a, optimal_b)
        } else {
            let optimal_a = amount_b * pool.reserve_a / pool.reserve_b;
            (optimal_a, amount_b)
        }
    };
    
    pool.add_liquidity(provider, optimal_a, optimal_b)
}

// Swap tokens
fn swap(
    pool: &mut LiquidityPool,
    user: &Address,
    token_in: &Address,
    amount_in: U256,
) -> Result<U256, SwapError> {
    if amount_in == U256::zero() {
        return Err(SwapError::ZeroAmount);
    }
    
    let amount_out = pool.swap(amount_in, token_in)?;
    
    if amount_out == U256::zero() {
        return Err(SwapError::InsufficientOutput);
    }
    
    Ok(amount_out)
}

// Remove liquidity
fn remove_liquidity(
    pool: &mut LiquidityPool,
    provider: &Address,
    shares: U256,
) -> (U256, U256) {
    let provider_shares = pool.shares.get(provider).copied().unwrap_or(U256::zero());
    
    if shares > provider_shares {
        panic!("Insufficient shares");
    }
    
    let amount_a = pool.reserve_a * shares / pool.total_shares;
    let amount_b = pool.reserve_b * shares / pool.total_shares;
    
    pool.reserve_a -= amount_a;
    pool.reserve_b -= amount_b;
    pool.total_shares -= shares;
    pool.shares.insert(*provider, provider_shares - shares);
    
    (amount_a, amount_b)
}
```

## Summary

Fusion provides a comprehensive toolkit for blockchain development:

- **Core Infrastructure**: Blocks, chains, transactions, and cryptographic primitives
- **Consensus**: PoW, PoS, DPoS, and PBFT with configurable parameters
- **Smart Contracts**: Full contract deployment, calls, and state management
- **Token Standards**: ERC-20, ERC-721, ERC-1155, and custom tokens
- **DeFi**: AMMs, lending protocols, and DEX functionality
- **Privacy**: Stealth addresses, confidential transactions, and zero-knowledge proofs
- **Governance**: On-chain proposals, voting, and DAOs
- **Networking**: P2P, peer discovery, chain synchronization, and gossip
- **Storage**: Block storage, state storage, and snapshots
- **Cross-Chain**: Bridges, messaging, and Layer 2 scaling

In the next chapter, we'll explore null/nil handling in Fusion, showing how the type system eliminates an entire class of bugs.
# F1r3fly Bitcoin Anchor Developer Guide

## Network Support

The F1r3fly Bitcoin Anchor system supports multiple Bitcoin networks for different development and testing scenarios:

- **Regtest** (Local Testnet) - Fully local testing environment with instant block generation
- **Signet** (Controlled Test Network) - Federated test network with predictable block times
- **Testnet3** - Public test network (planned support)  
- **Mainnet** - Bitcoin production network (planned support)

### Network Documentation

- **Regtest Setup**: Instructions below for local development
- **Signet Setup**: See [SIGNET.md](SIGNET.md) for detailed Signet integration guide

---

## Bitcoin Core - Regtest (Local Testnet)

### Setup
1. Install bitcoin-core: `brew install bitcoin`
2. Verify installation: `bitcoind --version`. `v29.0.0`
3. Create Bitcoin data directory and start local testnet:
   ```bash
   # Create data directory first
   mkdir -p ~/.bitcoin
   
   # Start bitcoind with custom data directory
   bitcoind -regtest -server -datadir=$HOME/.bitcoin \
     -rpcallowip=127.0.0.1 -rpcbind=127.0.0.1:18443 \
     -zmqpubrawblock=tcp://127.0.0.1:28332 -zmqpubrawtx=tcp://127.0.0.1:28333 \
     -fallbackfee=0.0001
   ```
   *Add `-daemon` to run in background*

4. Create wallet and mining address:
   ```bash
   bitcoin-cli -regtest -datadir=$HOME/.bitcoin createwallet "mining_wallet"
   bitcoin-cli -regtest -datadir=$HOME/.bitcoin getnewaddress "mining" "bech32"
   ```

5. Generate initial blocks and verify:
   ```bash
   bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress 101 <your-bcrt1q-address>
   bitcoin-cli -regtest -datadir=$HOME/.bitcoin getbalance  # Expected: 50.00000000
   ```
	 In Bitcoin, new coins from mining (called coinbase rewards) cannot be spent until they have 100 confirmations.

### Common Commands

#### Wallet Management
```bash
# List all available wallets in the node
bitcoin-cli -regtest -datadir=$HOME/.bitcoin listwallets

# Show all addresses that have received transactions (0 = include zero-confirmation, true = include empty addresses)
bitcoin-cli -regtest -datadir=$HOME/.bitcoin listreceivedbyaddress 0 true

# Send Bitcoin to a specific address with specified amount
bitcoin-cli -regtest -datadir=$HOME/.bitcoin sendtoaddress <address> <amount>

# Get current wallet balance
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getbalance

# Generate a new receiving address for the wallet
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getnewaddress "label" "bech32"
```

#### Mining & Blockchain Operations
```bash
# Generate blocks and send mining rewards to specified address (essential for regtest)
bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress 1 <miner_address>

# Get comprehensive blockchain information (height, difficulty, chain tips, etc.)
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getblockchaininfo

# Get detailed information about a specific block by hash
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getblock <block_hash>

# Get the hash of the block at a specific height
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getblockhash <block_height>

# Get memory pool information (pending transactions)
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getmempoolinfo
```

#### PSBT (Partially Signed Bitcoin Transaction) Operations
```bash
# Decode and display a PSBT file in human-readable format
bitcoin-cli -regtest -datadir=$HOME/.bitcoin decodepsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)"

# Process/sign a PSBT with the wallet's private keys
bitcoin-cli -regtest -datadir=$HOME/.bitcoin walletprocesspsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)"

# Finalize a PSBT (converts to raw transaction if fully signed)
bitcoin-cli -regtest -datadir=$HOME/.bitcoin finalizepsbt "<psbt_string>"
```

#### Transaction Operations
```bash
# Broadcast a raw transaction to the network
bitcoin-cli -regtest -datadir=$HOME/.bitcoin sendrawtransaction <hex_transaction>

# Get detailed information about a transaction by ID
bitcoin-cli -regtest -datadir=$HOME/.bitcoin gettransaction <txid>

# Get raw transaction data in hexadecimal format
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getrawtransaction <txid> true
```

#### Node Control
```bash
# Gracefully stop the Bitcoin node
bitcoin-cli -regtest -datadir=$HOME/.bitcoin stop

# Get node connection and network information
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getnetworkinfo

# Get peer connection information
bitcoin-cli -regtest -datadir=$HOME/.bitcoin getpeerinfo
```

## Electrs Backend

Repository: https://github.com/blockstream/electrs

### Installation & Setup

```bash
git clone https://github.com/blockstream/electrs && cd electrs
git checkout new-index

# macOS only: Set export flags before cargo build --release
export CPPFLAGS="-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/c++/v1"
export CXXFLAGS="-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/c++/v1"

cargo build --release
```

### Running Electrs (Regtest)

```bash
./target/release/electrs --network regtest --daemon-rpc-addr 127.0.0.1:18443 --electrum-rpc-addr 127.0.0.1:60401 --http-addr 0.0.0.0:3002 --monitoring-addr 127.0.0.1:24224 --db-dir ./db/regtest --daemon-dir ~/.bitcoin/ -vvv
```
Look for: `INFO REST server running on 0.0.0.0:3002`

## Esplora Frontend

Repository: https://github.com/Blockstream/esplora

### Installation & Setup

```bash
git clone https://github.com/Blockstream/esplora.git
cd esplora && npm i
```

### Configuration

```bash
cat > config.js << EOF
module.exports = {
  network: 'regtest',
  api_url: '/api',
  electrum_url: 'tcp://localhost:60401'
};
EOF
```

### Running Development Server

```bash
export API_URL=http://localhost:3002/ && PORT=5001 npm run dev-server
```

### Verify Setup
1. **Check Bitcoin Core is running on regtest:**
   ```bash
   bitcoin-cli -regtest -datadir=$HOME/.bitcoin getblockchaininfo
   ```

2. **Check Electrs is syncing:**
   Look for log messages showing block processing and indexing progress

3. **Access Esplora Frontend:**
   Open http://localhost:5001 in your browser

## F1r3fly Transaction Workflow

```bash
# 0. Ensure blocks are available
bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress 101 <your-address>

# 1. Create PSBT
cargo run --example create_f1r3fly_psbt_regtest -- <your-address>

# 2. Verify PSBT
bitcoin-cli -regtest -datadir=$HOME/.bitcoin decodepsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)"

# 3. Sign and broadcast transaction
SIGNED_RESULT=$(bitcoin-cli -regtest -datadir=$HOME/.bitcoin walletprocesspsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)") && \
RAW_TX=$(echo $SIGNED_RESULT | jq -r '.hex') && \
TXID=$(bitcoin-cli -regtest -datadir=$HOME/.bitcoin sendrawtransaction $RAW_TX) && \
echo "🎉 F1r3fly transaction broadcast!" && \
echo "Transaction ID: $TXID"

# 4. Generate block for confirmation
BLOCK_HASH=$(bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress 1 <your-address>) && \
echo "⚡ Block generated: $BLOCK_HASH"

# 5. Verify State Commitment
cargo run --example verify_f1r3fly_commitment -- <TXID>
``` 

## Troubleshooting

#### Coinbase Spending Error
**Error:** `bad-txns-premature-spend-of-coinbase, tried to spend coinbase at depth 44`

**Cause:** You're trying to spend a coinbase transaction (mining reward) that doesn't have enough confirmations. Bitcoin requires 100 confirmations before coinbase outputs can be spent.

**Resolution:** Generate more blocks to reach the required depth:
```bash
# If the error shows depth 44, you need 100 - 44 = 56 more blocks
bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress 56 <your-address>

# General formula: generate (100 - current_depth) blocks
bitcoin-cli -regtest -datadir=$HOME/.bitcoin generatetoaddress <100_minus_current_depth> <your-address>
```
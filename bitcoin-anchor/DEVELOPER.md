# F1r3fly Bitcoin Anchor Developer Guide

## Bitcoin Core - Regtest (Local Testnet)

### Setup
1. Install bitcoin-core: https://bitcoincore.org/en/download/ 
   
   *Or ask AI with this prompt:*
   ```
   How do I install Bitcoin Core via command line on [your OS]? 
   I need bitcoind and bitcoin-cli for development work.
   ```

2. Verify installation: `bitcoind --version`
3. Start local testnet:
   ```bash
   bitcoind -regtest -server -rpcallowip=127.0.0.1 -rpcbind=127.0.0.1:18443 \
     -zmqpubrawblock=tcp://127.0.0.1:28332 -zmqpubrawtx=tcp://127.0.0.1:28333 \
     -fallbackfee=0.0001
   ```
   *Add `-daemon` to run in background*

4. Create wallet and mining address:
   ```bash
   bitcoin-cli -regtest createwallet "mining_wallet"
   bitcoin-cli -regtest getnewaddress "mining" "bech32"
   ```

5. Generate initial blocks and verify:
   ```bash
   bitcoin-cli -regtest generatetoaddress 101 <your-bcrt1q-address>
   bitcoin-cli -regtest getbalance  # Expected: 50.00000000
   ```

### Common Commands
```bash
# Wallet management
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest listreceivedbyaddress 0 true
bitcoin-cli -regtest sendtoaddress <address> <amount>

# Mining & blockchain
bitcoin-cli -regtest generatetoaddress 1 <miner_address>
bitcoin-cli -regtest getblockchaininfo

# PSBT operations
bitcoin-cli -regtest decodepsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)"

# Stop node
bitcoin-cli -regtest stop
```

## Blockstream/Electrs

1. **Clone the repository:**
   ```bash
   git clone https://github.com/blockstream/electrs -b new-index
   cd electrs
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Start Electrs server:**
   ```bash
   ./target/release/electrs --network regtest \
     --daemon-rpc-addr 127.0.0.1:18443 \
     --electrum-rpc-addr 127.0.0.1:60401 \
     --http-addr 0.0.0.0:3002 \
     --monitoring-addr 127.0.0.1:24224 \
     --db-dir ./db/regtest \
     --daemon-dir ~/.bitcoin/ -vvv
   ```

4. **Verify server is running:**
   
   *Look for: `INFO REST server running on 0.0.0.0:3002`*

## Blockstream/Esplora

1. **Clone and install dependencies:**
   ```bash
   git clone https://github.com/Blockstream/esplora.git
   cd esplora && npm i
   ```

2. **Create configuration file:**
   ```bash
   cat > config.js << EOF
   module.exports = {
     network: 'regtest',
     api_url: '/api',
     electrum_url: 'tcp://localhost:60401'
   };
   EOF
   ```

3. **Set environment and start server:**
   ```bash
   export API_URL=http://localhost:3002/
   npm run dev-server
   ```

4. **Access the interface:**
   
   *Enable CORS extension for browser access: https://mybrowseraddon.com/access-control-allow-origin.html*  
   *Mempool interface: http://localhost:5000/mempool*

## F1r3fly Transaction Workflow

```bash
# 0. Ensure blocks are available
bitcoin-cli -regtest generatetoaddress 101 <your-address>

# 1. Create PSBT
cargo run --example create_f1r3fly_psbt_regtest -- <your-address>

# 2. Verify PSBT
bitcoin-cli -regtest decodepsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)"

# 3. Sign and broadcast transaction
SIGNED_RESULT=$(bitcoin-cli -regtest walletprocesspsbt "$(cat f1r3fly_transaction_<address_prefix>.psbt)") && \
RAW_TX=$(echo $SIGNED_RESULT | jq -r '.hex') && \
TXID=$(bitcoin-cli -regtest sendrawtransaction $RAW_TX) && \
echo "🎉 F1r3fly transaction broadcast!" && \
echo "Transaction ID: $TXID"

# 4. Generate block for confirmation
BLOCK_HASH=$(bitcoin-cli -regtest generatetoaddress 1 <your-address>) && \
echo "⚡ Block generated: $BLOCK_HASH" && \
echo "🔍 Checking transaction confirmation..."

# 5. Verify State Commitment
cargo run --example verify_f1r3fly_commitment -- <commitment_hash>
``` 
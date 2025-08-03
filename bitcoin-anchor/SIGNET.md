# F1r3fly Bitcoin Signet Integration Guide

This guide provides step-by-step instructions for using the F1r3fly Bitcoin Anchor system with Bitcoin Signet.

## Overview

Bitcoin Signet is a test network where blocks are signed by a federation rather than mined with proof-of-work. It provides a more controlled testing environment compared to testnet while still using real Bitcoin protocols.

## Prerequisites

### 1. Bitcoin Core (Signet Mode)

Install and start Bitcoin Core for Signet using command line options:

```bash
# Download Bitcoin Core from https://bitcoincore.org/en/download/

# Start Bitcoin Core in signet mode with RPC enabled (all via command line)
bitcoind -signet -server=1 -rpcuser=signetuser -rpcpassword=signetpass -rpcport=38332 -rpcbind=127.0.0.1 -rpcallowip=127.0.0.1 -prune=1000
```

**Wait for Bitcoin Core to start** (usually takes 15 minutes), then verify it's running:

```bash
# Test connection
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass getblockchaininfo

# Check these fields to ensure Bitcoin Core is syncing:
# - "chain": should be "signet"
# - "blocks": current block height
# - "headers": should be equal to or close to "blocks" when synced
# - "verificationprogress": should be close to 1.0 when fully synced
# - "initialblockdownload": should be false when synced
```

### 2. Signet Faucet Access

Get signet coins from a faucet (you'll use these in the workflow steps below):
- **Working Faucet**: https://alt.signetfaucet.com/

## Step-by-Step Workflow

### Step 1: Start Bitcoin Core and Verify Sync

Follow the Bitcoin Core setup instructions above, then verify it's syncing properly.

### Step 2: Create Wallet and Generate Signet Address

Create a wallet and generate signet addresses using Bitcoin Core:

**Note**: You can create the wallet and receive funds while Bitcoin Core is still syncing. You'll see confirmations once the node catches up to the relevant block height.

```bash
# Create a new descriptor wallet (modern format, avoids BDB deprecation warning)
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass createwallet "signet_wallet" false false "" false true true

# Generate a new signet address from the wallet
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass getnewaddress "" "bech32"

# Example output: tb1qxxx...xxx (signet addresses use tb1 prefix like testnet)
```

**Useful wallet commands:**
```bash
# List all wallets
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass listwallets

# Get wallet info
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass getwalletinfo

# List all addresses in the wallet
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass listreceivedbyaddress 0 true

# Backup wallet (important!)
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass backupwallet ~/signet_wallet_backup.dat
```

### Step 3: Fund the Address

1. Copy your generated signet address
2. Visit the signet faucet from the Prerequisites section: https://alt.signetfaucet.com/
3. Request signet coins to your address
4. Wait for confirmation (signet blocks are typically mined every 10 minutes)

Verify you received funds:

**Important**: Your Bitcoin Core node must be synced before you can see the balance. Check sync status first:
```bash
# Check if node is synced (verificationprogress should be close to 1.0)
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass getblockchaininfo

# Check address balance (only works if node is synced)
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass listunspent 0 9999999 '["tb1qyour_address_here"]'
```

### Step 4: Run F1r3fly Signet Example

Navigate to the bitcoin-anchor directory and run the signet example:

```bash
cd f1r3fly/bitcoin-anchor

# Run the signet PSBT creation example
cargo run --example create_f1r3fly_psbt_signet -- tb1qyour_funded_address_here

# Example output:
# 🌐 Using Bitcoin Signet network
# 💳 Address: tb1qxxx...xxx
# 🔍 Fetching UTXOs from Signet Esplora...
# ✅ Found 1 UTXO(s) totaling 50000000 sats
# 🏗️  Building PSBT for F1r3fly commitment...
# 📄 PSBT created: f1r3fly_transaction_signet_tb1qxxx.psbt
```

### Step 5: Inspect the PSBT (Optional but Recommended)

Before signing, you can inspect the PSBT to verify its contents:

```bash
# Decode and display the PSBT in human-readable format
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass decodepsbt "$(cat f1r3fly_transaction_signet_*.psbt)"

# This shows:
# - Input addresses and amounts
# - Output addresses and amounts
# - F1r3fly commitment data in OP_RETURN
# - Fee calculation
```

### Step 6: Sign the PSBT

Sign the generated PSBT using Bitcoin Core:

```bash
# Load and sign the PSBT, store the result
SIGN_RESULT=$(bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass walletprocesspsbt "$(cat f1r3fly_transaction_signet_*.psbt)")

# Display the result to see if transaction is complete
echo "$SIGN_RESULT" | jq '.'
```

### Step 7: Broadcast Transaction

Check if the transaction is complete and broadcast:

```bash
# Check if the transaction is complete (fully signed)
COMPLETE=$(echo "$SIGN_RESULT" | jq -r '.complete')

if [ "$COMPLETE" = "true" ]; then
  # Transaction is complete - extract the finalized hex and broadcast directly
  FINAL_TX=$(echo "$SIGN_RESULT" | jq -r '.hex')
  echo "✅ Transaction is fully signed, broadcasting..."
else
  # Transaction needs finalizing first
  echo "⚠️  Transaction incomplete, finalizing..."
  SIGNED_PSBT=$(echo "$SIGN_RESULT" | jq -r '.psbt')
  FINAL_TX=$(bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass finalizepsbt "$SIGNED_PSBT" | jq -r '.hex')
fi

# Broadcast to signet network
TXID=$(bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass sendrawtransaction "$FINAL_TX")
echo "🎉 Transaction broadcast! TXID: $TXID"
```

### Step 8: Verify the Commitment

**Important**: Wait for the transaction to be confirmed by the network before running verification. This usually takes 1-2 minutes for the transaction to propagate to the Esplora API.

```bash
# Check if transaction is confirmed (optional but recommended)
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass gettransaction "$TXID"

# Wait a moment for network propagation, then verify the F1r3fly commitment
cargo run --example verify_f1r3fly_commitment -- "$TXID"

# Example output:
# 🔍 Fetching transaction from Signet...
# ✅ Transaction found: abcd1234...
# 🔎 Scanning for F1r3fly commitment...
# ✅ F1r3fly commitment found!
# 💽 Commitment data: [32 bytes]
# ✅ Commitment verification successful!
```

**If verification fails immediately after broadcast**: Wait 1-2 minutes for the transaction to propagate to the public Esplora API, then try again.

## Fee Configuration (Optional)

If you want to adjust transaction fees for faster confirmation or cost optimization, you can modify the fee rate in the example code:

**File:** `examples/create_f1r3fly_psbt_signet.rs` (line 99)

```rust
// For faster confirmation (recommended):
let fee_rate = Some(50.0); // 50 sats/vbyte - fast confirmation

// Other options:
let fee_rate = Some(25.0); // 25 sats/vbyte - medium-high priority
let fee_rate = Some(100.0); // 100 sats/vbyte - very fast (high cost)

// Or use dynamic network estimation:
let fee_rate = None; // Uses current network fee estimates
```

After changing the fee rate, rebuild and run the example again:
```bash
cargo run --example create_f1r3fly_psbt_signet -- tb1qyour_address_here
```

## Stopping Bitcoin Core

When you're finished testing, stop Bitcoin Core gracefully:

```bash
# Stop Bitcoin Core signet daemon
bitcoin-cli -signet -rpcuser=signetuser -rpcpassword=signetpass stop

# Verify it's stopped (should show no bitcoin processes)
ps aux | grep bitcoind
```
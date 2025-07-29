# Validator4 Bonding Guide

This guide provides complete step-by-step instructions for bonding validator4 to the F1r3fly network and integrating it with the autopropose system.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Phase 1: Network Status Check](#phase-1-network-status-check)
3. [Phase 2: Start Validator4 Node](#phase-2-start-validator4-node)
4. [Phase 3: Fund Transfer (if needed)](#phase-3-fund-transfer-if-needed)
5. [Phase 4: Bond Validator4](#phase-4-bond-validator4)
6. [Phase 5: Monitor Quarantine Period](#phase-5-monitor-quarantine-period)
7. [Phase 6: Verify Active Participation](#phase-6-verify-active-participation)
8. [Phase 7: Configure Autopropose Integration](#phase-7-configure-autopropose-integration)
9. [Phase 8: Verify Block Proposing](#phase-8-verify-block-proposing)
10. [Troubleshooting](#troubleshooting)

## Prerequisites

- F1r3fly network running with existing validators
- Docker and docker-compose installed
- Node-cli built and functional
- Validator4 credentials available

### Network Credentials

#### Bootstrap Node
```
Private Key: 5f668a7ee96d944a4494cc947e4005e172d7ab3461ee5538f1f2a45a835e9657
Public Key:  04ffc016579a68050d655d55df4e09f04605164543e257c8e6df10361e6068a5336588e9b355ea859c5ab4285a5ef0efdf62bc28b80320ce99e26bb1607b3ad93d
ETH Address: fac7dde9d0fa1df6355bd1382fe75ba0c50e8840
REV Address: 1111AtahZeefej4tvVR6ti9TJtv8yxLebT31SCEVDCKMNikBk5r3g
```

#### Validator_1 (Active Validator)
```
Private Key: 357cdc4201a5650830e0bc5a03299a30038d9934ba4c7ab73ec164ad82471ff9
Public Key:  04fa70d7be5eb750e0915c0f6d19e7085d18bb1c22d030feb2a877ca2cd226d04438aa819359c56c720142fbc66e9da03a5ab960a3d8b75363a226b7c800f60420
ETH Address: a77c116ce0ebe1331487638233bb52ba6b277da7
REV Address: 111127RX5ZgiAdRaQy4AWy57RdvAAckdELReEBxzvWYVvdnR32PiHA
```

#### Validator_2 (Active Validator)
```
Private Key: 2c02138097d019d263c1d5383fcaddb1ba6416a0f4e64e3a617fe3af45b7851d
Public Key:  04837a4cff833e3157e3135d7b40b8e1f33c6e6b5a4342b9fc784230ca4c4f9d356f258debef56ad4984726d6ab3e7709e1632ef079b4bcd653db00b68b2df065f
ETH Address: df00c6395a23e9b2b8780de9a93c9522512947c3
REV Address: 111129p33f7vaRrpLqK8Nr35Y2aacAjrR5pd6PCzqcdrMuPHzymczH
```

#### Validator_3 (Active Validator)
```
Private Key: b67533f1f99c0ecaedb7d829e430b1c0e605bda10f339f65d5567cb5bd77cbcb
Public Key:  0457febafcc25dd34ca5e5c025cd445f60e5ea6918931a54eb8c3a204f51760248090b0c757c2bdad7b8c4dca757e109f8ef64737d90712724c8216c94b4ae661c
ETH Address: ca778c4ecf5c6eb285a86cedd4aaf5167f4eae13
REV Address: 1111LAd2PWaHsw84gxarNx99YVK2aZhCThhrPsWTV7cs1BPcvHftP
```

#### Validator_4 (Target for Bonding)
```
Private Key: 5ff3514bf79a7d18e8dd974c699678ba63b7762ce8d78c532346e52f0ad219cd
Public Key:  04d26c6103d7269773b943d7a9c456f9eb227e0d8b1fe30bccee4fca963f4446e3385d99f6386317f2c1ad36b9e6b0d5f97bb0a0041f05781c60a5ebca124a251d
ETH Address: 0cab9328d6d896e5159a1f70bc377e261ded7414
REV Address: 1111La6tHaCtGjRiv4wkffbTAAjGyMsVhzSUNzQxH1jjZH9jtEi3M
```

#### Autopropose Deploy Wallet
```
Private Key: 61e594124ca6af84a5468d98b34a4f3431ef39c54c6cf07fe6fbf8b079ef64f6
Public Key:  04a1f613710e2a4ac7a5fefa3c74ad97cbff42aefaed083d6134b913dba3e84857e698a88c23b0ae37668726a2e96c82cc724434ea165a7d0fd9d7cab71d5a8065
REV Address: 1111ocWgUJb5QqnYCvKiPtzcmMyfvD3gS5Eg84NtaLkUtRfw3TDS8
```

## Phase 1: Network Status Check

First, verify the current network state and understand the epoch configuration.

```bash
cd f1r3fly-build/node-cli

# Check current network health
cargo run -- network-health --standard-ports

# Get current epoch information (shows epoch & quarantine lengths)
cargo run -- epoch-info

# Check current network consensus
cargo run -- network-consensus

# Verify existing validators
cargo run -- active-validators --port 40413
cargo run -- bonds --port 40413
```

**Expected Output:**
- Network should show 3 active validators (validator1, validator2, validator3)
- Epoch length: 10 blocks, Quarantine length: 10 blocks (for testing)
- No validator4 in bonds or active validators list

## Phase 2: Start Validator4 Node

Start the validator4 node using docker-compose.

```bash
# Navigate to docker directory
cd ../docker

# Start validator4 node
docker-compose -f validator4.yml up -d

# Verify validator4 is running
docker ps | grep validator4

# Check validator4 logs (optional)
docker logs validator4

# Verify validator4 node is accessible
cd ../node-cli
cargo run -- status --port 40443  # validator4's HTTP port
```

**Expected Output:**
- Validator4 container should be running
- Status check should return node information

**Configuration Note:**
- Validator4 now uses the shared `shared-rnode.conf` configuration file (same as validator1-3)
- Validator-specific credentials are passed via environment variables (defined in `docker/.env`)
- Bootstrap connection and network settings are identical to other validators
- This simplifies configuration management and ensures consistency across all validators
- No need for separate `validator4.conf` file - everything uses the shared configuration!

## Phase 3: Fund Transfer (if needed)

Check if validator4 has sufficient REV for bonding (1000 REV required).

```bash
# Check validator4 REV balance
cargo run -- wallet-balance --address 1111La6tHaCtGjRiv4wkffbTAAjGyMsVhzSUNzQxH1jjZH9jtEi3M

# If balance is insufficient, transfer REV from bootstrap/validator1
cargo run -- transfer \
  --to-address 1111La6tHaCtGjRiv4wkffbTAAjGyMsVhzSUNzQxH1jjZH9jtEi3M \
  --amount 2000 \
  --private-key 5f668a7ee96d944a4494cc947e4005e172d7ab3461ee5538f1f2a45a835e9657 \
  --port 40412

# Verify transfer completed
cargo run -- wallet-balance --address 1111La6tHaCtGjRiv4wkffbTAAjGyMsVhzSUNzQxH1jjZH9jtEi3M
```

**Expected Output:**
- Validator4 should have at least 1000 REV for bonding

## Phase 4: Bond Validator4

Execute the bonding transaction to add validator4 to the network.

```bash
# Verify validator4 is NOT currently bonded
cargo run -- validator-status -k 04d26c6103d7269773b943d7a9c456f9eb227e0d8b1fe30bccee4fca963f4446e3385d99f6386317f2c1ad36b9e6b0d5f97bb0a0041f05781c60a5ebca124a251d

# Bond validator4 with 1000 REV stake
cargo run -- bond-validator \
  --stake 1000 \
  --private-key 5ff3514bf79a7d18e8dd974c699678ba63b7762ce8d78c532346e52f0ad219cd \
  --port 40442  # validator4's gRPC port

# Alternative: Bond with immediate block proposal
cargo run -- bond-validator \
  --stake 1000 \
  --private-key 5ff3514bf79a7d18e8dd974c699678ba63b7762ce8d78c532346e52f0ad219cd \
  --propose true \
  --port 40442
```

**Expected Output:**
- Bonding transaction should complete successfully
- Deploy ID should be returned

## Phase 5: Monitor Quarantine Period

Monitor validator4's transition from quarantine to active status.

```bash
# Check validator4 status immediately after bonding
cargo run -- validator-status -k 04d26c6103d7269773b943d7a9c456f9eb227e0d8b1fe30bccee4fca963f4446e3385d99f6386317f2c1ad36b9e6b0d5f97bb0a0041f05781c60a5ebca124a251d

# Start real-time monitoring (in separate terminal)
cargo run -- validator-transitions --watch --interval 10

# Monitor epoch progression
cargo run -- epoch-info

# Check network consensus status
cargo run -- network-consensus
```

**Expected Timeline (with testing config):**
- **Immediately after bonding**: Validator4 shows "⏳ QUARANTINE"
- **After ~10 blocks**: Validator4 transitions to "✅ ACTIVE"
- **Total wait time**: ~2-3 minutes (with 10-block quarantine)

**Expected Status Progression:**
1. `❌ NOT BONDED` → `✅ BONDED, ⏳ QUARANTINE` → `✅ BONDED, ✅ ACTIVE`

## Phase 6: Verify Active Participation

Confirm validator4 is now actively participating in consensus.

```bash
# Verify validator4 is active
cargo run -- validator-status -k 04d26c6103d7269773b943d7a9c456f9eb227e0d8b1fe30bccee4fca963f4446e3385d99f6386317f2c1ad36b9e6b0d5f97bb0a0041f05781c60a5ebca124a251d

# Check all active validators
cargo run -- active-validators --port 40413

# Verify network consensus health
cargo run -- network-consensus

# Check recent blocks
cargo run -- show-main-chain --depth 10 --port 40413
```

**Expected Output:**
- Validator4 status: "✅ ACTIVE: Validator is actively participating in consensus"
- Active validators: 4 total (validator1, validator2, validator3, validator4)
- Network consensus: "🟢 Healthy" with 4/4 active validators

## Phase 7: Configure Autopropose Integration

Add validator4 to the autopropose rotation system.

### 7.1 Update Autopropose Configuration

```bash
cd ../docker
```

Edit `autopropose/config.yml` to add validator4:

```yaml
validators:
  # Existing validators...
  - name: validator1
    host: rnode.validator1
    grpc_port: 40402
    enabled: true
    
  - name: validator2
    host: rnode.validator2
    grpc_port: 40402
    enabled: true
    
  - name: validator3
    host: rnode.validator3
    grpc_port: 40402
    enabled: true
  
  # Add Validator4
  - name: validator4
    host: rnode.validator4
    grpc_port: 40402
    enabled: true
    
  # Bootstrap (typically disabled)
  - name: bootstrap
    host: rnode.bootstrap
    grpc_port: 40402
    enabled: false
```

### 7.2 Restart Autopropose Service

```bash
# Restart autopropose to pick up new configuration
docker-compose -f shard-with-autopropose.yml restart autopropose

# Monitor autopropose logs
docker logs -f autopropose
```

**Expected Output:**
- Autopropose should show validator4 in the rotation
- Logs should indicate 4-validator rotation active

## Phase 8: Verify Block Proposing

Confirm validator4 is successfully proposing blocks in rotation.

```bash
# Monitor autopropose logs for validator4 activity
docker logs -f autopropose | grep validator4

# Check recent blocks for validator4 signatures
cd ../node-cli
cargo run -- show-main-chain --depth 20 --port 40413

# Monitor ongoing validator activity
cargo run -- validator-transitions --watch --interval 30

# Check network consensus continuously
cargo run -- network-consensus
```

**Success Indicators:**
- ✅ Autopropose logs show validator4 proposing blocks
- ✅ Recent blocks show validator4 as sender/proposer
- ✅ Network consensus remains healthy (4/4 active validators)
- ✅ Validator4 appears in regular rotation with other validators

## Troubleshooting

### Issue: Validator4 Stuck in Quarantine

**Symptoms:**
- Validator shows "⏳ QUARANTINE" for extended time
- No transition to active status

**Solutions:**
```bash
# Check current epoch timing
cargo run -- epoch-info

# Verify quarantine period
cargo run -- network-consensus

# Check if quarantine length needs adjustment in config
# Edit f1r3fly-build/docker/conf/shared-rnode.conf:
# quarantine-length = 10  # Reduce for faster testing
```

### Issue: Bonding Transaction Fails

**Symptoms:**
- Bond-validator command returns error
- Insufficient funds or network connection issues

**Solutions:**
```bash
# Verify validator4 node is running and accessible
cargo run -- status --port 40443

# Check validator4 REV balance
cargo run -- wallet-balance --address 1111La6tHaCtGjRiv4wkffbTAAjGyMsVhzSUNzQxH1jjZH9jtEi3M

# Ensure network connectivity
cargo run -- network-health --standard-ports

# Try different gRPC port or host
cargo run -- bond-validator --stake 1000 --private-key 5ff3514bf79a7d18e8dd974c699678ba63b7762ce8d78c532346e52f0ad219cd --port 40412
```

### Issue: Not Appearing in Active Validators

**Symptoms:**
- Validator4 shows bonded but not active
- Active validators list doesn't include validator4

**Solutions:**
```bash
# Monitor real-time transitions
cargo run -- validator-transitions --watch --interval 5

# Check for network consensus issues
cargo run -- network-consensus

# Verify epoch transitions are occurring
cargo run -- epoch-info

# Check if max validator limit reached
# Verify number-of-active-validators setting in config
```

### Issue: Autopropose Not Including Validator4

**Symptoms:**
- Validator4 is active but not proposing blocks
- Autopropose logs don't show validator4

**Solutions:**
```bash
# Verify autopropose configuration
cat ../docker/autopropose/config.yml

# Restart autopropose service
cd ../docker
docker-compose -f shard-with-autopropose.yml restart autopropose

# Check autopropose container logs
docker logs autopropose

# Verify validator4 network connectivity from autopropose
docker exec autopropose ping rnode.validator4
```

### Issue: Network Health Problems

**Symptoms:**
- Network consensus shows degraded status
- Block production issues

**Solutions:**
```bash
# Check overall network health
cargo run -- network-health --standard-ports

# Monitor all validator statuses
cargo run -- validator-transitions --watch --interval 10

# Check individual validator connectivity
cargo run -- status --port 40403  # bootstrap
cargo run -- status --port 40413  # validator1
cargo run -- status --port 40423  # validator2
cargo run -- status --port 40433  # validator3
cargo run -- status --port 40443  # validator4
```

## Summary

After completing this guide, you should have:

✅ **Validator4 bonded** to the network with 1000 REV stake
✅ **Active participation** in consensus (exited quarantine)
✅ **Autopropose integration** with 4-validator rotation
✅ **Block proposing** by validator4 in regular rotation
✅ **Network health** maintained with 4/4 active validators

The F1r3fly network now operates with 4 validators (validator1, validator2, validator3, validator4) in a robust, fault-tolerant configuration with automated block proposal rotation.
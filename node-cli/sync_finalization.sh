#!/bin/bash
# 
# Sync Finalization Script
# Forces all validators to re-validate from genesis to achieve consensus
# WITHOUT deleting blockchain data
#

set -e  # Exit on any error

# Get absolute paths to avoid directory confusion
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NODE_CLI_DIR="$SCRIPT_DIR"
DOCKER_DIR="$(cd "$SCRIPT_DIR/../docker" && pwd)"
CONF_DIR="$DOCKER_DIR/conf"

echo "🔧 ASI-Chain Finalization Sync Script"
echo "======================================"
echo "This script forces all validators to re-sync from genesis"
echo "to achieve finalized block consensus WITHOUT data loss."
echo ""

# Function to check if containers are running
check_containers() {
    echo "📊 Checking container status..."
    docker ps --format "table {{.Names}}\t{{.Status}}" | grep -E "(bootstrap|validator|finalizer)" || echo "No F1r3fly containers running"
    echo ""
}

# Function to get finalized block from all validators
check_finalization_status() {
    echo "🔍 Checking finalization status across all validators..."
    echo "======================================================"
    
    echo "Observer (40453):"
    $NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port 40453 || echo "❌ Observer not responding"
    echo ""
    
    echo "Validator1 (40413):"
    $NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port 40413 || echo "❌ Validator1 not responding"
    echo ""
    
    echo "Validator2 (40423):"
    $NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port 40423 || echo "❌ Validator2 not responding"
    echo ""
    
    echo "Validator3 (40433):"
    $NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port 40433 || echo "❌ Validator3 not responding"
    echo ""
    
    echo "Validator4 (40443):"
    $NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port 40443 || echo "❌ Validator4 not responding"
    echo ""
}

# Function to backup current configurations
backup_configs() {
    echo "💾 Backing up current configurations..."
    cp "$CONF_DIR/shared-rnode.conf" "$CONF_DIR/shared-rnode.conf.backup"
    for i in {1..4}; do
        if [ -f "$CONF_DIR/validator$i.conf" ]; then
            cp "$CONF_DIR/validator$i.conf" "$CONF_DIR/validator$i.conf.backup"
        fi
    done
    cp "$CONF_DIR/bootstrap.conf" "$CONF_DIR/bootstrap.conf.backup"
    echo "✅ Configurations backed up"
    echo ""
}

# Function to enable disable-lfs in all configs
enable_genesis_validation() {
    echo "⚙️  Enabling genesis validation (disable-lfs = true)..."
    
    # Update shared config
    sed -i.tmp 's/disable-lfs = false/disable-lfs = true/' "$CONF_DIR/shared-rnode.conf"
    
    # Update individual validator configs if they exist
    for i in {1..4}; do
        if [ -f "$CONF_DIR/validator$i.conf" ]; then
            sed -i.tmp 's/disable-lfs = false/disable-lfs = true/' "$CONF_DIR/validator$i.conf"
        fi
    done
    
    # Note: validator4 uses shared-rnode.conf, so it's already covered above
    
    # Update bootstrap config
    sed -i.tmp 's/disable-lfs = false/disable-lfs = true/' "$CONF_DIR/bootstrap.conf"
    
    # Clean up temp files
    rm -f "$CONF_DIR"/*.tmp
    
    echo "✅ All validators will now re-validate from genesis"
    echo ""
}

# Function to restore original configurations
restore_configs() {
    echo "🔄 Restoring original configurations..."
    
    mv "$CONF_DIR/shared-rnode.conf.backup" "$CONF_DIR/shared-rnode.conf"
    for i in {1..4}; do
        if [ -f "$CONF_DIR/validator$i.conf.backup" ]; then
            mv "$CONF_DIR/validator$i.conf.backup" "$CONF_DIR/validator$i.conf"
        fi
    done
    mv "$CONF_DIR/bootstrap.conf.backup" "$CONF_DIR/bootstrap.conf"
    
    echo "✅ Original configurations restored"
    echo ""
}

# Function to wait for consensus
wait_for_consensus() {
    echo "⏳ Waiting for validators to achieve consensus..."
    echo "This may take several minutes as validators re-validate from genesis."
    echo ""
    
    local max_wait=300  # 5 minutes
    local wait_time=0
    local consensus_achieved=false
    
    while [ $wait_time -lt $max_wait ] && [ "$consensus_achieved" = false ]; do
        echo "🔄 Checking consensus status (${wait_time}s elapsed)..."
        
        # Try to get finalized blocks from all validators
        local blocks=()
        local success_count=0
        
        for port in 40413 40423 40433 40443; do
            local result
            result=$($NODE_CLI_DIR/target/debug/node_cli last-finalized-block --port $port 2>/dev/null | grep "Block Number:" | grep -o '[0-9]\+' || echo "FAIL")
            if [ "$result" != "FAIL" ]; then
                blocks+=("$result")
                success_count=$((success_count + 1))
            fi
        done
        
        # Check if we have consensus (all responding validators agree)
        if [ $success_count -ge 3 ]; then
            # Get unique block numbers
            local unique_blocks=($(printf '%s\n' "${blocks[@]}" | sort -u))
            if [ ${#unique_blocks[@]} -eq 1 ]; then
                echo "✅ Consensus achieved! All validators agree on block ${unique_blocks[0]}"
                consensus_achieved=true
            else
                echo "⏳ Validators still syncing... (blocks: ${blocks[*]})"
            fi
        else
            echo "⏳ Waiting for more validators to respond ($success_count/4)..."
        fi
        
        if [ "$consensus_achieved" = false ]; then
            sleep 10
            wait_time=$((wait_time + 10))
        fi
    done
    
    if [ "$consensus_achieved" = false ]; then
        echo "⚠️  Consensus not achieved within $max_wait seconds"
        echo "You may need to wait longer or check validator logs"
    fi
    echo ""
}

# Main execution
main() {
    echo "Step 1: Initial status check"
    check_containers
    echo "Press Enter to continue or Ctrl+C to abort..."
    read
    
    echo "Step 2: Checking current finalization status"
    check_finalization_status
    echo "Press Enter to continue with sync process..."
    read
    
    echo "Step 3: Stopping autopropose..."
    cd "$DOCKER_DIR"
    docker stop finalizer-bot || echo "Finalizer not running"
    echo ""
    
    echo "Step 4: Stopping shard and validator4..."
    docker-compose -f shard-with-autopropose.yml down
    docker stop rnode.validator4 || echo "Validator4 not running"
    echo ""
    
    echo "Step 5: Backing up and modifying configurations..."
    backup_configs
    enable_genesis_validation
    
    echo "Step 6: Starting shard with genesis re-validation..."
    docker-compose -f shard-with-autopropose.yml up -d
    echo "Starting validator4 separately..."
    docker-compose -f validator4.yml up -d || echo "Validator4 config not found - skipping"
    echo ""
    
    echo "Step 7: Waiting for validators to be ready..."
    sleep 30
    echo ""
    
    echo "Step 8: Building node-cli..."
    cd "$NODE_CLI_DIR"
    cargo build --release
    echo ""
    
    echo "Step 9: Waiting for consensus..."
    wait_for_consensus
    
    echo "Step 10: Restoring original configurations..."
    cd "$DOCKER_DIR"
    docker-compose -f shard-with-autopropose.yml down
    restore_configs
    
    echo "Step 11: Restarting with normal configuration..."
    docker-compose -f shard-with-autopropose.yml up -d
    echo "Restarting validator4..."
    docker-compose -f validator4.yml up -d || echo "Validator4 config not found - skipping"
    sleep 30
    echo ""
    
    echo "Step 12: Final verification..."
    cd "$NODE_CLI_DIR"
    check_finalization_status
    
    echo "Step 13: Restarting autopropose..."
    cd "$DOCKER_DIR"
    docker-compose -f shard-with-autopropose.yml up finalizer-bot -d
    echo ""
    
    echo "🎉 Finalization sync complete!"
    echo "All validators should now have consistent finalized state."
    echo ""
    echo "Monitor autopropose with:"
    echo "  docker logs -f finalizer-bot"
}

# Run main function
main "$@"
#!/bin/bash

set -euo pipefail

#================================================================================================
# RChain Node Management Script
# Description: Script to run and manage RChain nodes without docker
#================================================================================================

# Constants and Configuration
readonly SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Node types
readonly NODE_TYPES=("bootstrap" "validator1" "validator2" "validator3" "observer")

# File paths
readonly ENV_FILE=".env"
readonly RUST_LIBRARIES_PATH="../rust_libraries/release"
readonly JAR_FILE="../node/target/scala-2.12/rnode-assembly-1.0.0-SNAPSHOT.jar"

# Java configuration
readonly JAVA_OPTS="--add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED -Djna.library.path=$RUST_LIBRARIES_PATH -Dlogback.configurationFile=./conf/logback.xml"

# Common node configurations
readonly COMMON_ARGS="--host=127.0.0.1 --allow-private-addresses --no-upnp --bonds-file=./genesis/bonds.txt --wallets-file=./genesis/wallets.txt"

#================================================================================================
# Utility Functions
#================================================================================================

# Logging functions
log_info() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [INFO] $*"
}

log_warn() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [WARN] $*" >&2
}

log_error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [ERROR] $*" >&2
}

log_success() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [SUCCESS] $*"
}

# Print section separator
print_separator() {
    echo "================================================================"
}

# Load environment variables from .env file
load_environment() {
    if [[ -f "$ENV_FILE" ]]; then
        log_info "Loading environment variables from $ENV_FILE"
        set -a
        # shellcheck source=/dev/null
        source "$ENV_FILE"
        set +a
        log_success "Environment variables loaded successfully"
    else
        log_warn "Environment file $ENV_FILE not found"
    fi
}

# Check if node is already running
is_node_running() {
    local node_name="$1"
    local pid_file="${node_name}.pid"
    
    if [[ -f "$pid_file" ]]; then
        local pid
        pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            return 0  # Running
        else
            log_warn "Stale PID file found for $node_name (PID: $pid), removing..."
            rm -f "$pid_file"
            return 1  # Not running
        fi
    fi
    return 1  # Not running
}

# Get node configuration
get_node_config() {
    local node_name="$1"
    
    case "$node_name" in
        "bootstrap")
            echo "--config-file=./conf/bootstrap-ceremony.conf"
            ;;
        *)
            echo "--config-file=./conf/shared-rnode.conf"
            ;;
    esac
}

# Initialize environment
load_environment
export RUST_LIBRARIES_PATH JAR_FILE JAVA_OPTS



#================================================================================================
# Node Management Functions
#================================================================================================

# Run a specific node type
run_node() {
    local node_name="$1"
    local pid_file="${node_name}.pid"
    local log_file="${node_name}.log"
    
    log_info "Attempting to start $node_name node..."
    
    if is_node_running "$node_name"; then
        log_warn "$node_name node is already running"
        return 0
    fi

    if [[ ! -d "./data/" ]]; then
        mkdir -p "./data/"
    fi
    
    log_info "Starting $node_name node..."
    
    # Execute the specific run command for this node type
    case "$node_name" in
        "bootstrap")
            run_bootstrap_command
            ;;
        "validator1")
            run_validator1_command
            ;;
        "validator2")
            run_validator2_command
            ;;
        "validator3")
            run_validator3_command
            ;;
        "observer")
            run_observer_command
            ;;
        *)
            log_error "Unknown node type: $node_name"
            return 1
            ;;
    esac
    
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        log_success "$node_name node started successfully (PID: $(cat "$pid_file" 2>/dev/null || echo "unknown"))"
        log_info "Log file: $log_file"
    else
        log_error "Failed to start $node_name node"
        return $exit_code
    fi
}

# Bootstrap node specific command (keeping exact original command)
run_bootstrap_command() {
    java $JAVA_OPTS -jar $JAR_FILE run \
        --config-file=./conf/bootstrap-ceremony.conf \
        $COMMON_ARGS \
        --validator-private-key=$BOOTSTRAP_PRIVATE_KEY \
        --validator-public-key=$BOOTSTRAP_PUBLIC_KEY \
        --protocol-port=40400 \
        --api-port-grpc-external=40401 \
        --api-port-grpc-internal=40402 \
        --api-port-http=40403 \
        --discovery-port=40404 \
        --api-port-admin-http=40405 \
        --data-dir=./data/$BOOTSTRAP_HOST \
        --tls-certificate-path=./certs/bootstrap/node.certificate.pem \
        --tls-key-path=./certs/bootstrap/node.key.pem > ./bootstrap.log 2>&1 &
    echo $! > bootstrap.pid
}

# Validator1 node specific command (keeping exact original command)
run_validator1_command() {
    java $JAVA_OPTS -jar $JAR_FILE run \
        --config-file=./conf/shared-rnode.conf \
        $COMMON_ARGS \
        --validator-private-key=$VALIDATOR1_PRIVATE_KEY \
        --validator-public-key=$VALIDATOR1_PUBLIC_KEY \
        --protocol-port=40410 \
        --api-port-grpc-external=40411 \
        --api-port-grpc-internal=40412 \
        --api-port-http=40413 \
        --discovery-port=40414 \
        --api-port-admin-http=40415 \
        --data-dir=./data/$VALIDATOR1_HOST \
        --tls-certificate-path=./certs/validator1/node.certificate.pem \
        --tls-key-path=./certs/validator1/node.key.pem \
        --genesis-validator \
        --bootstrap="rnode://$BOOTSTRAP_NODE_ID@127.0.0.1?protocol=40400&discovery=40404" > ./validator1.log 2>&1 &
    echo $! > validator1.pid
}

# Validator2 node specific command (keeping exact original command)
run_validator2_command() {
    java $JAVA_OPTS -jar $JAR_FILE run \
        --config-file=./conf/shared-rnode.conf \
        $COMMON_ARGS \
        --validator-private-key=$VALIDATOR2_PRIVATE_KEY \
        --validator-public-key=$VALIDATOR2_PUBLIC_KEY \
        --protocol-port=40420 \
        --api-port-grpc-external=40421 \
        --api-port-grpc-internal=40422 \
        --api-port-http=40423 \
        --discovery-port=40424 \
        --api-port-admin-http=40425 \
        --data-dir=./data/$VALIDATOR2_HOST \
        --tls-certificate-path=./certs/validator2/node.certificate.pem \
        --tls-key-path=./certs/validator2/node.key.pem \
        --genesis-validator \
        --bootstrap="rnode://$BOOTSTRAP_NODE_ID@127.0.0.1?protocol=40400&discovery=40404" > ./validator2.log 2>&1 &
    echo $! > validator2.pid
}

# Validator3 node specific command (keeping exact original command)
run_validator3_command() {
    java $JAVA_OPTS -jar $JAR_FILE run \
        --config-file=./conf/shared-rnode.conf \
        $COMMON_ARGS \
        --validator-private-key=$VALIDATOR3_PRIVATE_KEY \
        --validator-public-key=$VALIDATOR3_PUBLIC_KEY \
        --protocol-port=40430 \
        --api-port-grpc-external=40431 \
        --api-port-grpc-internal=40432 \
        --api-port-http=40433 \
        --discovery-port=40434 \
        --api-port-admin-http=40435 \
        --data-dir=./data/$VALIDATOR3_HOST \
        --tls-certificate-path=./certs/validator3/node.certificate.pem \
        --tls-key-path=./certs/validator3/node.key.pem \
        --bootstrap="rnode://$BOOTSTRAP_NODE_ID@127.0.0.1?protocol=40400&discovery=40404" \
        --genesis-validator > ./validator3.log 2>&1 &
    echo $! > validator3.pid
}

# Observer node specific command (keeping exact original command)
run_observer_command() {
    java $JAVA_OPTS -jar $JAR_FILE run \
        --config-file=./conf/shared-rnode.conf \
        $COMMON_ARGS \
        --protocol-port=40440 \
        --api-port-grpc-external=40441 \
        --api-port-grpc-internal=40442 \
        --api-port-http=40443 \
        --discovery-port=40444 \
        --api-port-admin-http=40445 \
        --data-dir=./data/observer \
        --bootstrap="rnode://$BOOTSTRAP_NODE_ID@127.0.0.1?protocol=40400&discovery=40404" > ./observer.log 2>&1 &
    echo $! > observer.pid
}

# Wrapper functions for backward compatibility
run_bootstrap() {
    run_node "bootstrap"
}

run_validator1() {
    run_node "validator1"
}

run_validator2() {
    run_node "validator2"
}

run_validator3() {
    run_node "validator3"
}

run_observer() {
    run_node "observer"
}

#================================================================================================
# Stop Functions
#================================================================================================

# Stop a specific node
stop_node() {
    local node_name="$1"
    local pid_file="${node_name}.pid"
    local log_file="${node_name}.log"
    
    log_info "Attempting to stop $node_name node..."
    
    if [[ ! -f "$pid_file" ]]; then
        log_warn "$node_name node is not running (no PID file found)"
        return 0
    fi
    
    local pid
    pid=$(cat "$pid_file")
    
    if kill -0 "$pid" 2>/dev/null; then
        log_info "Stopping $node_name node (PID: $pid)..."
        if kill -TERM "$pid" 2>/dev/null; then
            # Wait for graceful shutdown
            local count=0
            while kill -0 "$pid" 2>/dev/null && [[ $count -lt 10 ]]; do
                sleep 1
                ((count++))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                log_warn "Graceful shutdown timeout, force killing $node_name node..."
                kill -KILL "$pid" 2>/dev/null || true
            fi
        fi
        
        # Verify the process is stopped
        if ! kill -0 "$pid" 2>/dev/null; then
            log_success "$node_name node stopped successfully"
        else
            log_error "Failed to stop $node_name node"
            return 1
        fi
    else
        log_warn "$node_name node PID file exists but process is not running"
    fi
    
    # Clean up files
    rm -f "$pid_file" "$log_file"
    log_info "Cleaned up $node_name node files"
}

# Clean data directory for a specific node
clean_node_data() {
    local node_name="$1"
    local data_dir
    
    case "$node_name" in
        "bootstrap")
            data_dir="./data/$BOOTSTRAP_HOST"
            ;;
        "validator1")
            data_dir="./data/$VALIDATOR1_HOST"
            ;;
        "validator2")
            data_dir="./data/$VALIDATOR2_HOST"
            ;;
        "validator3")
            data_dir="./data/$VALIDATOR3_HOST"
            ;;
        "observer")
            data_dir="./data/observer"
            ;;
        *)
            log_error "Unknown node type: $node_name"
            return 1
            ;;
    esac
    
    if [[ -d "$data_dir" ]]; then
        log_info "Cleaning $node_name data directory: $data_dir"
        rm -rf "$data_dir"
        log_success "$node_name data directory cleaned"
    else
        log_info "$node_name data directory does not exist: $data_dir"
    fi
}

# Stop all nodes and clean everything
stop_all() {
    log_info "Initiating complete shutdown of all nodes..."
    print_separator
    
    # Stop individual nodes
    for node in "${NODE_TYPES[@]}"; do
        stop_node "$node"
    done
    
    print_separator
    log_info "Checking for remaining Java processes..."
    
    # Kill any remaining Java processes
    local java_pids
    java_pids=$(pgrep -f "java.*$JAR_FILE" 2>/dev/null || true)
    
    if [[ -n "$java_pids" ]]; then
        log_warn "Found remaining Java processes, force killing..."
        echo "$java_pids" | while read -r pid; do
            if [[ -n "$pid" ]]; then
                log_info "Force killing process: $pid"
                kill -KILL "$pid" 2>/dev/null || true
            fi
        done
    else
        log_info "No remaining Java processes found"
    fi
    
    print_separator
    log_info "Cleaning data directories..."
    
    # Clean data directories
    for node in "${NODE_TYPES[@]}"; do
        clean_node_data "$node"
    done
    
    # Clean up any remaining log and pid files
    log_info "Cleaning remaining log and PID files..."
    for node in "${NODE_TYPES[@]}"; do
        rm -f "${node}.log" "${node}.pid"
    done
    
    print_separator
    log_success "Complete shutdown and cleanup finished successfully"
}

#================================================================================================
# Information and Validation Functions
#================================================================================================

# Get node status
get_node_status() {
    local node_name="$1"
    local pid_file="${node_name}.pid"
    
    if [[ ! -f "$pid_file" ]]; then
        echo "STOPPED"
        return 1
    fi
    
    local pid
    pid=$(cat "$pid_file")
    
    if kill -0 "$pid" 2>/dev/null; then
        echo "RUNNING"
        return 0
    else
        echo "DEAD"
        return 1
    fi
}

# List all nodes with their status
list_nodes() {
    log_info "Listing all node statuses..."
    print_separator
    
    printf "%-12s %-8s %-10s %-15s\n" "NODE" "PID" "STATUS" "PROCESS"
    printf "%-12s %-8s %-10s %-15s\n" "------------" "--------" "----------" "---------------"
    
    for node in "${NODE_TYPES[@]}"; do
        local pid_file="${node}.pid"
        local pid=""
        local status=""
        local process_name=""
        
        if [[ -f "$pid_file" ]]; then
            pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                status="RUNNING"
                process_name=$(ps -p "$pid" -o comm= 2>/dev/null || echo "unknown")
            else
                status="DEAD"
                process_name="N/A"
            fi
        else
            status="STOPPED"
            pid="N/A"
            process_name="N/A"
        fi
        
        # Capitalize first letter for display (shell-compatible way)
        local display_name="$(echo "${node:0:1}" | tr '[:lower:]' '[:upper:]')${node:1}"
        printf "%-12s %-8s %-10s %-15s\n" "$display_name" "$pid" "$status" "$process_name"
    done
    
    print_separator
    log_info "Checking for additional Java processes..."
    
    local additional_pids
    additional_pids=$(pgrep -f "java.*$JAR_FILE" 2>/dev/null || true)
    
    if [[ -n "$additional_pids" ]]; then
        echo "Additional Java processes found:"
        echo "$additional_pids" | while read -r pid; do
            if [[ -n "$pid" ]]; then
                local cmd
                cmd=$(ps -p "$pid" -o args= 2>/dev/null | cut -c1-80 || echo "unknown")
                printf "PID: %-8s Command: %s\n" "$pid" "$cmd"
            fi
        done
    else
        log_info "No additional Java processes found"
    fi
    
    print_separator
}

# Validate prerequisites
validate_prerequisites() {
    local errors=0
    
    log_info "Validating prerequisites..."
    # Check JAR file
    if [[ -f "$JAR_FILE" ]]; then
        local jar_size
        jar_size=$(ls -lh "$JAR_FILE" | awk '{print $5}')
        log_success "JAR file found: $JAR_FILE (size: $jar_size)"
    else
        log_error "JAR file not found: $JAR_FILE"
        log_info "Run '$SCRIPT_NAME -B' to build the JAR file"
        ((errors++))
    fi
    
    # Check Rust libraries
    if [[ -d "$RUST_LIBRARIES_PATH" ]]; then
        local rust_size
        rust_size=$(du -h "$RUST_LIBRARIES_PATH" | tail -n 1 | awk '{print $1}')
        log_success "Rust libraries found: $RUST_LIBRARIES_PATH (size: $rust_size)"
    else
        log_error "Rust libraries not found: $RUST_LIBRARIES_PATH"
        log_info "Run '$SCRIPT_NAME -B [project_folder]' to build the Rust libraries"
        ((errors++))
    fi
    
    # Check environment variables
    local required_vars=("BOOTSTRAP_PRIVATE_KEY" "BOOTSTRAP_PUBLIC_KEY" "BOOTSTRAP_NODE_ID" "BOOTSTRAP_HOST")
    required_vars+=("VALIDATOR1_PRIVATE_KEY" "VALIDATOR1_PUBLIC_KEY" "VALIDATOR1_HOST")
    required_vars+=("VALIDATOR2_PRIVATE_KEY" "VALIDATOR2_PUBLIC_KEY" "VALIDATOR2_HOST")
    required_vars+=("VALIDATOR3_PRIVATE_KEY" "VALIDATOR3_PUBLIC_KEY" "VALIDATOR3_HOST")
    
    local missing_vars=()
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        log_error "Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            log_error "  - $var"
        done
        log_info "Please check your $ENV_FILE file"
        ((errors++))
    else
        log_success "All required environment variables are set"
    fi
    
    if [[ $errors -gt 0 ]]; then
        log_error "Validation failed with $errors error(s)"
        return 1
    else
        log_success "All prerequisites validated successfully"
        return 0
    fi
}

#================================================================================================
# Build Function
#================================================================================================

# Build the JAR file and rust libraries
build_project() {
    log_info "Running SBT build..."
    cd ../
    if sbt 'clean; compile; project node; assembly'; then
        log_success "SBT build completed successfully"
        cd -
        print_separator
        log_success "Build completed successfully"
        return 0
    else
        log_error "SBT build failed"
        cd "$original_dir"
        return 1
    fi
}

# Show help information
show_help() {
    cat << EOF
$SCRIPT_NAME - RChain Node Management Script

DESCRIPTION:
    Script to run and manage RChain nodes without docker.

USAGE:
    $SCRIPT_NAME [OPTION]

REQUIREMENTS:
    - Location: ./docker/ in root of the project
    - Java 17 or later
    - JAR file: $JAR_FILE
    - Rust libraries: $RUST_LIBRARIES_PATH
    - Environment variables in $ENV_FILE

CURRENT STATUS:
EOF
    validate_prerequisites
    
    cat << EOF

OPTIONS:
    Node Management:
      -b,  --bootstrap     Run bootstrap node
      -v1, --validator1    Run validator1 node
      -v2, --validator2    Run validator2 node
      -v3, --validator3    Run validator3 node
      -o,  --observer      Run observer node
      -s,  --stop          Stop all nodes and clean data
      -l,  --list          List all nodes with status
    
    Build and Maintenance:
      -B,  --build         Build JAR file and rust libraries from source
      -c,  --check         Validate prerequisites only
    
    Information:
      -h,  --help          Display this help message

EXAMPLES:
    $SCRIPT_NAME -b                    # Start bootstrap node
    $SCRIPT_NAME -v1                   # Start validator1 node
    $SCRIPT_NAME -l                    # List all node statuses
    $SCRIPT_NAME -s                    # Stop all nodes
    $SCRIPT_NAME -B                    # Build from parent directory
    $SCRIPT_NAME -c                    # Validate prerequisites

For more information, visit: https://rchain.coop
EOF
}

#================================================================================================
# Command Line Argument Parsing
#================================================================================================

# Parse command line arguments
parse_arguments() {
    if [[ $# -eq 0 ]]; then
        log_warn "No arguments provided"
        show_help
        exit 1
    fi
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -s|--stop)
                log_info "Stopping all nodes..."
                stop_all
                ;;
            -b|--bootstrap)
                run_bootstrap
                ;;
            -v1|--validator1)
                run_validator1
                ;;
            -v2|--validator2)
                run_validator2
                ;;
            -v3|--validator3)
                run_validator3
                ;;
            -o|--observer)
                run_observer
                ;;
            -l|--list)
                list_nodes
                ;;
            -B|--build)
                build_project
                ;;
            -c|--check)
                validate_prerequisites
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                log_info "Use -h or --help for usage information"
                exit 1
                ;;
        esac
        shift
    done
}

#================================================================================================
# Main Execution
#================================================================================================

# Main function
main() {
    log_info "RChain Node Management Script started"
    log_info "Working directory: $(pwd)"
    
    parse_arguments "$@"
}

# Execute main function with all arguments
main "$@"

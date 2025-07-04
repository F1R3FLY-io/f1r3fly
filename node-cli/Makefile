# F1r3fly Node CLI Makefile

EXAMPLE_FILE := ../rholang/examples/stdout.rho

# Build the CLI
build:
	cargo build

# Run the CLI with default example file
deploy:
	cargo run -- deploy -f $(EXAMPLE_FILE)

# Deploy with bigger phlo limit
deploy-big:
	cargo run -- deploy -f $(EXAMPLE_FILE) -b

# Propose a block
propose:
	cargo run -- propose

# Deploy and propose in one operation
full-deploy:
	cargo run -- full-deploy -f $(EXAMPLE_FILE)

# Full deploy with bigger phlo limit
full-deploy-big:
	cargo run -- full-deploy -f $(EXAMPLE_FILE) -b

# Check if a block is finalized (requires a block hash)
check-finalized:
	@echo "Usage: make check-finalized BLOCK_HASH=<block hash>"
	@[ "$(BLOCK_HASH)" ] && cargo run -- is-finalized --block-hash $(BLOCK_HASH) || echo "Please provide a block hash: make check-finalized BLOCK_HASH=<block hash>"

# Execute Rholang code without committing to the blockchain (exploratory deployment)
exploratory-deploy:
	cargo run -- exploratory-deploy -f $(EXAMPLE_FILE)

# Execute exploratory deployment with specific block hash
exploratory-deploy-at-block:
	@echo "Usage: make exploratory-deploy-at-block BLOCK_HASH=<block hash>"
	@[ "$(BLOCK_HASH)" ] && cargo run -- exploratory-deploy -f $(EXAMPLE_FILE) --block-hash $(BLOCK_HASH) || echo "Please provide a block hash: make exploratory-deploy-at-block BLOCK_HASH=<block hash>"

# Generate public key from private key
generate-key:
	@if [ "$(PRIVATE_KEY)" ]; then \
		if [ "$(COMPRESSED)" = "true" ]; then \
			cargo run -- generate-public-key --private-key $(PRIVATE_KEY) --compressed; \
		else \
			cargo run -- generate-public-key --private-key $(PRIVATE_KEY); \
		fi; \
	else \
		cargo run -- generate-public-key; \
	fi

# Generate a new private/public key pair
generate-key-pair:
	@if [ "$(SAVE)" = "true" ]; then \
		if [ "$(OUTPUT_DIR)" ]; then \
			if [ "$(COMPRESSED)" = "true" ]; then \
				cargo run -- generate-key-pair --save --output-dir $(OUTPUT_DIR) --compressed; \
			else \
				cargo run -- generate-key-pair --save --output-dir $(OUTPUT_DIR); \
			fi; \
		else \
			if [ "$(COMPRESSED)" = "true" ]; then \
				cargo run -- generate-key-pair --save --compressed; \
			else \
				cargo run -- generate-key-pair --save; \
			fi; \
		fi; \
	else \
		if [ "$(COMPRESSED)" = "true" ]; then \
			cargo run -- generate-key-pair --compressed; \
		else \
			cargo run -- generate-key-pair; \
		fi; \
	fi

# Node inspection commands

# Get node status
status:
	cargo run -- status

# Get recent blocks
blocks:
	cargo run -- blocks

# Get validator bonds
bonds:
	cargo run -- bonds

# Get active validators
active-validators:
	cargo run -- active-validators

# Check wallet balance (requires ADDRESS parameter)
wallet-balance:
	@echo "Usage: make wallet-balance ADDRESS=<wallet address>"
	@[ "$(ADDRESS)" ] && cargo run -- wallet-balance --address $(ADDRESS) || echo "Please provide a wallet address: make wallet-balance ADDRESS=<address>"

# Check bond status (requires PUBLIC_KEY parameter)
bond-status:
	@echo "Usage: make bond-status PUBLIC_KEY=<public key>"
	@[ "$(PUBLIC_KEY)" ] && cargo run -- bond-status --public-key $(PUBLIC_KEY) || echo "Please provide a public key: make bond-status PUBLIC_KEY=<public_key>"

# Get node metrics
metrics:
	cargo run -- metrics

# Get last finalized block
last-finalized-block:
	cargo run -- last-finalized-block

# Dynamic validator addition commands

# Bond a new validator (optionally STAKE)
bond-validator:
	@echo "Usage: make bond-validator [STAKE=<stake amount>]"
	@if [ "$(STAKE)" ]; then \
		cargo run -- bond-validator --stake $(STAKE); \
	else \
		cargo run -- bond-validator; \
	fi

# Check network health across multiple nodes
network-health:
	cargo run -- network-health

# Check network health with custom ports
network-health-custom:
	@echo "Usage: make network-health-custom CUSTOM_PORTS=<port1,port2,port3>"
	@[ "$(CUSTOM_PORTS)" ] && cargo run -- network-health --custom-ports $(CUSTOM_PORTS) || echo "Please provide custom ports: make network-health-custom CUSTOM_PORTS=<port1,port2,port3>"

# Help message
help:
	@echo "F1r3fly Node CLI Makefile"
	@echo ""
	@echo "Available commands:"
	@echo "  make build          - Build the CLI"
	@echo "  make deploy         - Deploy example Rholang file"
	@echo "  make deploy-big     - Deploy with bigger phlo limit"
	@echo "  make propose        - Propose a block"
	@echo "  make full-deploy    - Deploy and propose in one operation"
	@echo "  make full-deploy-big - Full deploy with bigger phlo limit"
	@echo "  make check-finalized BLOCK_HASH=<hash> - Check if a block is finalized"
	@echo "  make exploratory-deploy - Execute Rholang without committing (read-only mode)"
	@echo "  make exploratory-deploy-at-block BLOCK_HASH=<hash> - Execute Rholang at a specific block"
	@echo "  make generate-key [PRIVATE_KEY=<key>] [COMPRESSED=true] - Generate public key from private key"
	@echo "  make generate-key-pair [COMPRESSED=true] [SAVE=true] [OUTPUT_DIR=<dir>] - Generate new key pair"
	@echo ""
	@echo "Node inspection commands:"
	@echo "  make status         - Get node status and peer information"
	@echo "  make blocks         - Get recent blocks"
	@echo "  make bonds          - Get current validator bonds"
	@echo "  make active-validators - Get active validators"
	@echo "  make wallet-balance ADDRESS=<address> - Check wallet balance"
	@echo "  make bond-status PUBLIC_KEY=<key> - Check validator bond status"
	@echo "  make metrics        - Get node metrics"
	@echo "  make last-finalized-block - Get last finalized block"
	@echo ""
	@echo "Dynamic validator addition commands:"
	@echo "  make bond-validator [STAKE=<amount>] - Bond a new validator to the network (deploy only)"
	@echo "  make network-health - Check health across standard F1r3fly shard ports"
	@echo "  make network-health-custom CUSTOM_PORTS=<port1,port2> - Check health on custom ports"

.PHONY: build deploy deploy-big propose full-deploy full-deploy-big check-finalized exploratory-deploy exploratory-deploy-at-block generate-key generate-key-pair status blocks bonds active-validators wallet-balance bond-status metrics last-finalized-block bond-validator network-health network-health-custom help 
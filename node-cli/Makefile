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

.PHONY: build deploy deploy-big propose full-deploy full-deploy-big check-finalized exploratory-deploy exploratory-deploy-at-block generate-key generate-key-pair help 
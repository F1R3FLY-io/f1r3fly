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

.PHONY: build deploy deploy-big propose full-deploy full-deploy-big help 
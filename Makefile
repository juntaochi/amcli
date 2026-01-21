.PHONY: build install clean test run fmt lint help verify

# Variables
BINARY_NAME=amcli
CARGO=cargo

# Build the application
build:
	@echo "Building $(BINARY_NAME)..."
	$(CARGO) build --release

# Install the application
install:
	@echo "Installing $(BINARY_NAME)..."
	$(CARGO) install --path .

# Clean build artifacts
clean:
	@echo "Cleaning..."
	$(CARGO) clean
	@rm -rf bin/
	@rm -rf dist/

# Run tests
test:
	@echo "Running tests..."
	$(CARGO) test

# Run the application
run:
	$(CARGO) run

# Format code
fmt:
	@echo "Formatting code..."
	$(CARGO) fmt

# Lint code
lint:
	@echo "Linting code..."
	$(CARGO) clippy -- -D warnings

# Verify the project (runs script)
verify:
	@./scripts/verify.sh

# Show help
help:
	@echo "Available targets:"
	@echo "  build    - Build the application"
	@echo "  install  - Install the application"
	@echo "  clean    - Clean build artifacts"
	@echo "  test     - Run tests"
	@echo "  run      - Run the application"
	@echo "  fmt      - Format code"
	@echo "  lint     - Lint code"
	@echo "  verify   - Run full verification script"
	@echo "  help     - Show this help message"

.DEFAULT_GOAL := help

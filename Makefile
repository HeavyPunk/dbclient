# Makefile for Rust project

# Variables
CARGO = cargo
INSTALL_PATH = /usr/local/bin
BINARY_NAME = dbclient
TARGET_DIR = target/release

# Default target
.PHONY: all
all: build

# Build the project in release mode
.PHONY: build
build:
	$(CARGO) build --release

# Build in debug mode
.PHONY: debug
debug:
	$(CARGO) build

# Run tests
.PHONY: test
test:
	$(CARGO) test

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean

# Install the binary to system
.PHONY: install
install: build
	@echo "Installing $(BINARY_NAME) to $(INSTALL_PATH)"
	@sudo install -m 755 $(TARGET_DIR)/$(BINARY_NAME) $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "Installation complete!"

# Uninstall the binary from system
.PHONY: uninstall
uninstall:
	@echo "Removing $(BINARY_NAME) from $(INSTALL_PATH)"
	@sudo rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "Uninstallation complete!"

# Format code
.PHONY: fmt
fmt:
	$(CARGO) fmt

# Run clippy
.PHONY: clippy
clippy:
	$(CARGO) clippy -- -D warnings

# Build and run
.PHONY: run
run:
	$(CARGO) run --release -- --config-path ./test.config.toml

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make build    - Build the project in release mode"
	@echo "  make debug    - Build the project in debug mode"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install  - Build and install to system"
	@echo "  make uninstall- Remove from system"
	@echo "  make fmt      - Format code"
	@echo "  make clippy   - Run clippy linter"
	@echo "  make run      - Build and run"

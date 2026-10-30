# Makefile for NucleusFlow
# Works on macOS and Linux without modification.

.PHONY: all check clippy test fmt lint deny audit doc examples clean install uninstall verify-versions help

# Default single validation gate
all: check clippy test

check: ## Check the project for errors without producing outputs
	@cargo check --all-targets

clippy: ## Lint the project with Clippy (zero warnings allowed)
	@cargo clippy --all-targets -- -D warnings

test: ## Run unit, integration, and property tests
	@cargo test --all-features

fmt: ## Check code formatting
	@cargo fmt --all -- --check

lint: clippy ## Alias for clippy

deny: ## Run cargo deny supply-chain checks
	@cargo deny check

audit: ## Run cargo audit advisory check
	@cargo audit --no-fetch

doc: ## Build documentation with rustdoc
	@cargo doc --no-deps --all-features

examples: ## Run basic examples
	@cargo run --example basic

verify-versions: ## Verify that release version numbers are synchronized
	@./scripts/verify-release-versions.sh

install: ## Install binary, manpage, and completions using Unix contract
	@$(MAKE) -f GNUmakefile install

uninstall: ## Uninstall binary, manpage, and completions
	@$(MAKE) -f GNUmakefile uninstall

clean: ## Clean build artifacts
	@cargo clean

help: ## Display this help message
	@echo "Usage: make [target]..."
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

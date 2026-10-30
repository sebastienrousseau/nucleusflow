# SPDX-License-Identifier: Apache-2.0 OR MIT
# GNUmakefile implementing the standard Unix install contract for NucleusFlow.

PACKAGE = nucleusflow
PREFIX ?= /usr/local
DESTDIR ?=
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
MANDIR = $(DATADIR)/man/man1
BASHCOMPDIR = $(DATADIR)/bash-completion/completions
ZSHCOMPDIR = $(DATADIR)/zsh/site-functions
FISHCOMPDIR = $(DATADIR)/fish/vendor_completions.d

TARGET_BIN = target/release/$(PACKAGE)

.PHONY: all build test check clippy fmt lint deny audit doc examples clean install uninstall verify-versions help

all: build

build:
	cargo build --release --bin $(PACKAGE)

test:
	cargo test --all-features

check:
	cargo check --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

lint: clippy

deny:
	cargo deny check

audit:
	cargo audit --no-fetch

doc:
	cargo doc --no-deps --all-features

examples:
	cargo run --example basic

verify-versions:
	@./scripts/verify-release-versions.sh

install: build
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 "$(TARGET_BIN)" "$(DESTDIR)$(BINDIR)/$(PACKAGE)"
	install -d "$(DESTDIR)$(MANDIR)"
	install -m 644 share/man/man1/$(PACKAGE).1 "$(DESTDIR)$(MANDIR)/$(PACKAGE).1"
	install -d "$(DESTDIR)$(BASHCOMPDIR)"
	install -m 644 completions/bash/$(PACKAGE).bash "$(DESTDIR)$(BASHCOMPDIR)/$(PACKAGE)"
	install -d "$(DESTDIR)$(ZSHCOMPDIR)"
	install -m 644 completions/zsh/_$(PACKAGE) "$(DESTDIR)$(ZSHCOMPDIR)/_$(PACKAGE)"
	install -d "$(DESTDIR)$(FISHCOMPDIR)"
	install -m 644 completions/fish/$(PACKAGE).fish "$(DESTDIR)$(FISHCOMPDIR)/$(PACKAGE).fish"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/$(PACKAGE)"
	rm -f "$(DESTDIR)$(MANDIR)/$(PACKAGE).1"
	rm -f "$(DESTDIR)$(BASHCOMPDIR)/$(PACKAGE)"
	rm -f "$(DESTDIR)$(ZSHCOMPDIR)/_$(PACKAGE)"
	rm -f "$(DESTDIR)$(FISHCOMPDIR)/$(PACKAGE).fish"

clean:
	cargo clean

help:
	@echo "Usage: make [target]..."
	@echo ""
	@echo "Targets:"
	@echo "  build            Build release binary"
	@echo "  test             Run test suite"
	@echo "  check            Run cargo check"
	@echo "  clippy           Run clippy with -D warnings"
	@echo "  fmt              Check code formatting"
	@echo "  deny             Run cargo deny supply-chain checks"
	@echo "  audit            Run cargo audit advisory check"
	@echo "  doc              Build documentation"
	@echo "  examples         Run basic example"
	@echo "  verify-versions  Verify version synchronization"
	@echo "  install          Install binary, manpage, and completions"
	@echo "  uninstall        Uninstall installed files"
	@echo "  clean            Clean build artifacts"

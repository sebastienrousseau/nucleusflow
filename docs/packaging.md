# Packaging and Distribution Guide for Maintainers

This document provides instructions for Linux, BSD, macOS, and container packagers distributing NucleusFlow.

## Metadata Summary

- **Package Name**: `nucleusflow`
- **License**: `Apache-2.0 OR MIT` (SPDX standard identifiers)
- **MSRV**: `Rust 1.89.0`
- **Upstream Repository**: <https://github.com/sebastienrousseau/nucleusflow>
- **Release Signing**: Annotated git tags and release assets signed by Sebastien Rousseau (`KEYS.asc`).

## Build Contract

NucleusFlow adheres to standard GNU/FHS install contracts via `GNUmakefile` in addition to native Cargo workflows:

```bash
# Standard Unix compilation and staging
make -f GNUmakefile PREFIX=/usr DESTDIR=/tmp/stage install
```

### Installed Files & Target Paths

| Source | Target Path | Description |
| :--- | :--- | :--- |
| `target/release/nucleusflow` | `$(PREFIX)/bin/nucleusflow` | Main executable binary (mode 0755) |
| `share/man/man1/nucleusflow.1` | `$(PREFIX)/share/man/man1/nucleusflow.1` | Manual page (mode 0644) |
| `completions/bash/nucleusflow.bash` | `$(PREFIX)/share/bash-completion/completions/nucleusflow` | Bash completion |
| `completions/zsh/_nucleusflow` | `$(PREFIX)/share/zsh/site-functions/_nucleusflow` | Zsh completion |
| `completions/fish/nucleusflow.fish` | `$(PREFIX)/share/fish/vendor_completions.d/nucleusflow.fish` | Fish completion |

## Offline / Vendored Builds

For distributions requiring completely offline builds (Debian, Fedora, Alpine):

```bash
# 1. Vendor dependencies
cargo vendor vendor

# 2. Configure offline build
mkdir -p .cargo
cat <<EOF > .cargo/config.toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# 3. Build offline
cargo build --frozen --offline --release --bin nucleusflow

# 4. Run tests offline
cargo test --frozen --offline
```

## Security & Verification

Verify upstream release tarballs using published SHA-256 digests or GPG signatures:

```bash
# Verify checksums
sha256sum -c <<< "<expected-sha256>  nucleusflow-<version>-<target>.tar.gz"

# Import public keys
gpg --import KEYS.asc
```

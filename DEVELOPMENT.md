<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Developing NucleusFlow

The single entry point for developers and contributors working on NucleusFlow. User-facing documentation lives in the [README](README.md) and [`docs/`](docs/); contribution guidelines live in [`CONTRIBUTING.md`](CONTRIBUTING.md). This file details the toolchain, tasks, and local reproduction of every CI gate.

## Toolchain

| Tool | Version | Purpose |
| :--- | :--- | :--- |
| Rust stable | `1.89.0` or later | MSRV, enforced by `rust-version` in `Cargo.toml` and CI |
| Rust nightly | recent | Miri, cargo-fuzz, and code coverage (`cargo-llvm-cov`) |
| cargo-deny | latest | License, ban, and advisory validation |
| cargo-audit | latest | Known vulnerability scanning against RustSec DB |
| clippy / rustfmt | component | Static analysis and formatting enforcement |

```bash
git clone https://github.com/sebastienrousseau/nucleusflow.git
cd nucleusflow
make # default gate: check, clippy, test
```

## Task map

| Task | Command | Description |
| :--- | :--- | :--- |
| Single validation gate | `make` | Runs check, clippy, and test suite |
| Full test suite | `make test` | Runs unit, integration, and property tests |
| Linter | `make lint` | Runs `cargo clippy --all-targets -- -D warnings` |
| Format check | `make fmt` | Runs `cargo fmt --check` |
| Supply chain check | `make deny` | Runs `cargo deny check` |
| Advisory audit | `make audit` | Runs `cargo audit` |
| Documentation build | `make doc` | Builds rustdoc documentation with warnings denied |
| Clean artifacts | `make clean` | Cleans cargo target directory |

## Reproducing CI gates locally

NucleusFlow enforces strict verification on every pull request targeting `main`:

1. **Format and lint**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   ```
2. **Test suite and property testing**:
   ```bash
   cargo test --all-features
   ```
3. **Supply chain and license audit**:
   ```bash
   cargo deny check
   cargo audit
   ```
4. **Package build verification**:
   ```bash
   cargo package
   ```

## Test layout

- `src/**` `#[cfg(test)]`: Unit tests located next to source logic.
- `tests/property_tests.rs`: Property-based state space exploration and in-memory concurrency testing powered by `proptest`.
- `examples/`: Runnable examples demonstrating library integration.

## Release model

Releases increment strictly by `+0.0.1`. Work begins on a `feat/v<next-version>` branch. Before tagging:
- Ensure zero open Dependabot alerts and zero open issues.
- Verify `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `CITATION.cff`, and `README.md` versions agree.
- Tag cryptographically with `git tag -s -a v<VERSION> -m "NucleusFlow <VERSION>"`.
- GitHub release title must strictly be `NucleusFlow <VERSION>` (without leading `v`).

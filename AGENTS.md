<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Working on NucleusFlow with an AI agent

Invariants and rules for AI-assisted development on NucleusFlow. `DEVELOPMENT.md` covers the developer workflow; this document defines the rules that an agent must respect.

## What NucleusFlow is

NucleusFlow is a high-performance, memory-safe static site generator and content processing pipeline written in Rust. It prioritises security with `#![forbid(unsafe_code)]`, strict input sanitisation, path traversal rejection, and deterministic content generation.

## Versioning and branches

- Releases increment strictly by **+0.0.1**. Never propose a 0.1.0 or 1.0 jump without progressing through 0.0.999.
- Work lands on a `feat/v<next-version>` branch named for the release it targets, sequenced by commits.
- There is at most ONE active pull request targeting the default branch (`main`). All topic work and dependency updates funnel into this release branch.
- Commit or stash everything before any branch switch, and never run blind bulk additions without checking `git status` first.

## Quality gates

- CI must be green in the same session that turned it red.
- Zero-unsafe policy: `#![forbid(unsafe_code)]` is strictly enforced across the crate and binary roots.
- Every behaviour change lands with tests in the same commit. Input parsing and path sanitisation must be accompanied by property tests (`proptest`).
- Run the local battery before pushing:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - `cargo deny check`
  - `cargo audit`
- A change to path sanitisation, frontmatter parsing, or HTML generation that alters output format is a breaking change.

## Style and Git standards

- Conventional commits (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).
- Header maximum 50 characters, body wrapped strictly at 72 characters explaining WHAT and WHY.
- Zero em dashes anywhere in commit messages, pull requests, release notes, or documentation; use commas, colons, parentheses, or spaced hyphens.
- Commits and tags are signed; commits carry the human developer sign-off (`Signed-off-by:`) and agent attribution trailer (`Assisted-by:`).
- No tool attribution footers in pull request descriptions.
- Pull request descriptions strictly follow `/Users/seb/Code/PR-TEMPLATE.md`.

## Handover policy

- Handover commands left for human execution must be written to a self-contained executable script at `/tmp/nucleusflow/<script-name>.sh`.
- Provide the single execution command: `zsh /tmp/nucleusflow/<script-name>.sh`.

## Off-limits without explicit maintainer direction

- Merging any pull request into `main` or protected branches.
- Force pushes, tag deletion, or re-tagging.
- Publishing to crates.io or any external registry.
- Changing MSRV or weakening security boundaries.

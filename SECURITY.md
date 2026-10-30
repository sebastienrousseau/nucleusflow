<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :---: |
| 0.0.x | Yes |

## Reporting a Vulnerability

Report security vulnerabilities directly by emailing **sebastian.rousseau@gmail.com**.

Do not open a public issue for security reports.

Include:
- A description of the vulnerability.
- Steps to reproduce the issue.
- Affected versions and configurations.
- Proof-of-concept or exploit demonstration (if available).
- Any suggested fix or remediation guidance.

Expect an initial response within 48 hours. A remediation or coordinated disclosure plan will follow within 7 days of confirmation.

## Security Architecture

NucleusFlow is engineered from the ground up for strict reliability and safe static content generation:

- **Zero-Unsafe Guarantee**: Enforces `#![forbid(unsafe_code)]` at both crate library and binary roots. All memory operations are proven sound by the Rust compiler.
- **Path Traversal Sanitisation**: All input paths, templates, and content assets are validated and sanitised via `core::config::sanitize_path`. Empty paths, parent directory traversal (`..`), absolute paths where relative are expected, and path escapes outside the canonical root are strictly rejected with typed `ConfigurationError` errors.
- **Bounded DoS Mitigation**: File size validation prevents unbounded memory consumption during batch generation.
- **Pure-Rust Dependency Tree**: Relies on pure-Rust parsing and generation with zero C-FFI or unverified binary bindings.

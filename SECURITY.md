<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :---: |
| 0.0.x | Yes |

## Reporting a Vulnerability

Report security vulnerabilities directly by emailing **sebastian.rousseau@gmail.com** or submitting a report through GitHub Private Vulnerability Reporting at:
<https://github.com/sebastienrousseau/nucleusflow/security/advisories/new>

Do not open a public issue or discussion for security reports.

Please include:
- A description of the vulnerability and potential impact.
- Steps to reproduce the issue.
- Affected versions and configurations.
- Proof-of-concept or exploit demonstration (if available).
- Any suggested fix or remediation guidance.

### Disclosure Process and Timelines

- **Initial Response**: Expect an acknowledgement within 48 hours.
- **Triage and Assessment**: Confirmation and severity assessment within 5 business days.
- **Remediation**: A fix and coordinated disclosure release will be prepared within 14 days of confirmation.
- **Security Advisories**: Advisories will be published via GitHub Security Advisories at <https://github.com/sebastienrousseau/nucleusflow/security/advisories>.

## Security Architecture

NucleusFlow is engineered from the ground up for strict reliability and safe static content generation:

- **Zero-Unsafe Guarantee**: Enforces `#![forbid(unsafe_code)]` at both crate library and binary roots. All memory operations are proven sound by the Rust compiler.
- **Path Traversal Sanitisation**: All input paths, templates, and content assets are validated and sanitised via `core::config::sanitize_path`. Empty paths, parent directory traversal (`..`), absolute paths where relative are expected, and path escapes outside the canonical root are strictly rejected with typed `ConfigurationError` errors.
- **Bounded DoS Mitigation**: File size validation prevents unbounded memory consumption during batch generation.
- **Pure-Rust Dependency Tree**: Relies on pure-Rust parsing and generation with zero C-FFI or unverified binary bindings.

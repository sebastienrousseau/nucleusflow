#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -euo pipefail

# Verify that all version references in the repository are strictly synchronized.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
CITATION_VERSION=$(grep '^version: ' CITATION.cff | awk '{print $2}')
README_SNIPPET_VERSION=$(grep -m1 '^nucleusflow = ' README.md | cut -d '"' -f2)

echo "==> Verifying repository release version synchronization..."
echo "    Cargo.toml version:        ${CARGO_VERSION}"
echo "    CITATION.cff version:      ${CITATION_VERSION}"
echo "    README.md snippet version: ${README_SNIPPET_VERSION}"

FAILED=0

if [ "${CARGO_VERSION}" != "${CITATION_VERSION}" ]; then
  echo "ERROR: CITATION.cff version (${CITATION_VERSION}) does not match Cargo.toml (${CARGO_VERSION})"
  FAILED=1
fi

if [ "${CARGO_VERSION}" != "${README_SNIPPET_VERSION}" ]; then
  echo "ERROR: README.md dependency version (${README_SNIPPET_VERSION}) does not match Cargo.toml (${CARGO_VERSION})"
  FAILED=1
fi

if [ "${FAILED}" -ne 0 ]; then
  echo "==> Version verification FAILED."
  exit 1
fi

echo "==> All release versions are strictly synchronized: ${CARGO_VERSION}"

# syntax=docker/dockerfile:1.7
# Multi-stage minimal container build for NucleusFlow
# SPDX-License-Identifier: Apache-2.0 OR MIT

FROM rust:1.89-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends make && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY share/ ./share/
COPY completions/ ./completions/
COPY GNUmakefile ./

RUN cargo build --release --bin nucleusflow

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /site

COPY --from=builder /app/target/release/nucleusflow /usr/local/bin/nucleusflow

USER nonroot:nonroot
ENTRYPOINT ["nucleusflow"]
CMD ["--help"]

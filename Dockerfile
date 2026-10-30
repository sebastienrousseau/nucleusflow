# syntax=docker/dockerfile:1.7
# Multi-stage minimal container build for NucleusFlow
# SPDX-License-Identifier: Apache-2.0 OR MIT

FROM rust:1.89-slim-bookworm@sha256:d7fc7de78bb8c1469933aeecbf801314d30d7d6e9f0578bba4cfa285bfa37fe6 AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends make && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY share/ ./share/
COPY completions/ ./completions/
COPY GNUmakefile ./

RUN cargo build --release --bin nucleusflow

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
WORKDIR /site

COPY --from=builder /app/target/release/nucleusflow /usr/local/bin/nucleusflow

USER nonroot:nonroot
ENTRYPOINT ["nucleusflow"]
CMD ["--help"]

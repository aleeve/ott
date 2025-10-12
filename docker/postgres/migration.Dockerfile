FROM rust:1 AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR /workspace

FROM chef AS planner
COPY crates .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
ARG CRATE_NAME
COPY --from=planner /workspace/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY crates .
RUN cargo build --release --bin ${CRATE_NAME}

FROM debian:trixie-slim AS runtime
ARG CRATE_NAME
# Gah, time in my docker vm is wrong
RUN apt-get update -o Acquire::Max-FutureTime=31536000 --allow-insecure-repositories --allow-releaseinfo-change-suite  &&\
    apt-get install -y postgresql-client && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/${CRATE_NAME} app

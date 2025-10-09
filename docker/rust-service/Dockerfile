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

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
ARG CRATE_NAME
WORKDIR /workspace
COPY --from=builder /workspace/target/release/${CRATE_NAME} /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]

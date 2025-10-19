FROM rust:1 AS chef 
RUN cargo install cargo-chef 
WORKDIR /workspace


FROM chef AS planner
COPY crates .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
COPY crates/Cargo.* .
COPY crates/**/Cargo.toml .
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY crates .
ARG CRATE_NAME
RUN cargo build --release --bin ${CRATE_NAME}

FROM debian:trixie-slim AS runtime
RUN apt-get update &&\
    apt-get install -y ca-certificates &&\
    rm -rf /var/lib/apt/lists/*

ARG CRATE_NAME
ENV USER=${CRATE_NAME}
RUN useradd \
  --create-home \
  --home-dir "/home/$USER" \
  --shell /bin/bash \
  "$USER"
USER $USER
WORKDIR /home/${CRATE_NAME}
COPY ./connectors/fluvio_profile.toml /home/${CRATE_NAME}/.fluvio/config
COPY --from=builder /workspace/target/release/${CRATE_NAME} /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]

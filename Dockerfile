FROM rust:1.89-trixie AS chef

# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY crates/subgraph  ./crates/subgraph
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY crates/subgraph  ./crates/subgraph
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN cargo build --release --bins

# We do not need the Rust toolchain to run the binary!
FROM debian:trixie-slim AS runtime
WORKDIR app

# Install curl for healthchecks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

ARG BINARY_NAME
ENV BINARY_NAME=${BINARY_NAME}
COPY --from=builder /app/target/release/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}
ENTRYPOINT ["sh", "-c", "exec \"/usr/local/bin/${BINARY_NAME}\""]

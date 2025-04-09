FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app
# Install clang and wasm build tools
RUN apt-get update && apt-get install -y clang lld llvm pkg-config && rm -rf /var/lib/apt/lists/*

RUN cargo install dioxus-cli
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies for both native and wasm
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Use wasm-pack for web builds to handle wasm target properly
RUN dx build --platform web --no-default-features
RUN cargo build --release --bin gitlucky

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/gitlucky /usr/local/bin
COPY --from=builder /app/target/dx/gitlucky/debug/web/public /app/target/dx/gitlucky/debug/web/public
ENTRYPOINT ["/usr/local/bin/gitlucky"]
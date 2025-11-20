# Use the official Rust image with all build tools
FROM rust:1.91-slim-bookworm as builder

# Install system dependencies required for Sui compilation
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    llvm-dev \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/suiverify-indexer /usr/local/bin/suiverify-indexer

# Set the startup command
CMD ["suiverify-indexer", "--remote-store-url", "https://checkpoints.testnet.sui.io", "--first-checkpoint", "251834555"]

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
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Find and set libclang path
RUN LIBCLANG_PATH=$(find /usr/lib -name "libclang.so*" | head -n 1 | xargs dirname) && \
    echo "Found libclang at: $LIBCLANG_PATH" && \
    echo "export LIBCLANG_PATH=$LIBCLANG_PATH" >> /etc/profile

# Set environment variables for bindgen
ENV LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application with explicit LIBCLANG_PATH
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

# Use the official Rust image with all build tools
FROM rust:1.91-slim-bookworm as builder

# Install system dependencies required for Sui compilation
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    llvm \
    llvm-dev \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Verify libclang installation and set path
RUN echo "Searching for libclang..." && \
    find /usr -name "libclang.so*" 2>/dev/null && \
    LIBCLANG_SO=$(find /usr/lib -name "libclang.so*" 2>/dev/null | grep -v "\.a$" | head -n 1) && \
    if [ -z "$LIBCLANG_SO" ]; then \
        echo "ERROR: libclang.so not found!" && exit 1; \
    fi && \
    LIBCLANG_DIR=$(dirname "$LIBCLANG_SO") && \
    echo "Found libclang at: $LIBCLANG_DIR" && \
    echo "LIBCLANG_PATH=$LIBCLANG_DIR" >> /etc/environment

# Set environment variables
ENV LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/llvm-14/lib/clang/14.0.6/include"

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

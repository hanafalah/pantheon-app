# Multi-stage Dockerfile for Pantheon App

# ============================================================================
# Stage 1: Builder
# ============================================================================
FROM rust:1.75-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml ./
COPY rust-support/Cargo.toml ./rust-support/
COPY projects/pantheon-business/Cargo.toml ./projects/pantheon-business/ 2>/dev/null || true

# Create dummy source files to build dependencies (for caching)
RUN mkdir -p rust-support/src && \
    echo "fn main() {}" > rust-support/src/lib.rs
RUN mkdir -p projects/pantheon-business/src && \
    echo "fn main() {}" > projects/pantheon-business/src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release || true
RUN rm -rf rust-support/src projects/pantheon-business/src

# Copy actual source code
COPY . .

# Build application
RUN cargo build --release --package pantheon-business

# ============================================================================
# Stage 2: Runtime
# ============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 pantheon

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/pantheon-business /app/pantheon-business

# Copy configuration files
COPY --from=builder /usr/src/app/config /app/config

# Set ownership
RUN chown -R pantheon:pantheon /app

# Switch to app user
USER pantheon

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run the application
CMD ["./pantheon-business"]

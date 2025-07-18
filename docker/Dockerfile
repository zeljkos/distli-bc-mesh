FROM rust:latest as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Since build context is project root (..), copy everything
COPY . .

# Verify the project structure
RUN echo "=== Project structure ===" && \
    ls -la && \
    echo "=== Source structure ===" && \
    find src -name "*.rs" | head -10 && \
    echo "=== Enterprise BC files ===" && \
    ls -la src/enterprise_bc/ && \
    echo "=== Cargo.toml binaries ===" && \
    grep -A 3 "\[\[bin\]\]" Cargo.toml

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from build stage (excluding dashboard)
COPY --from=builder /app/target/release/enterprise-validator /app/
COPY --from=builder /app/target/release/tracker /app/

# Copy public files if they exist
COPY --from=builder /app/public /app/public

# Create data directory and set permissions
RUN mkdir -p /app/data && \
    chmod +x /app/enterprise-validator /app/tracker

CMD ["./enterprise-validator"]

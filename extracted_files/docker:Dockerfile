FROM rust:1.70 as builder

WORKDIR /app

# Copy the entire project
COPY . .

# Build all binaries
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all binaries from builder
COPY --from=builder /app/target/release/tracker /app/tracker
COPY --from=builder /app/target/release/enterprise-validator /app/enterprise-validator
COPY --from=builder /app/target/release/enterprise-dashboard /app/enterprise-dashboard

# Copy source for cargo run (development mode)
COPY --from=builder /app /app

# Create data directory
RUN mkdir -p /app/data

# Expose ports
EXPOSE 3030 8080 9090

# Default command (can be overridden)
CMD ["./enterprise-validator", "--id", "validator1", "--port", "8080"]

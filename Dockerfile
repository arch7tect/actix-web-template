# Multi-stage Dockerfile for Actix Web Memos Application

# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and resources
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migration ./migration

# Build for release (build all workspace members)
RUN cargo build --release --all

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        libpq5 \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binaries from builder (workspace builds to shared target/)
COPY --from=builder /app/target/release/actix-web-template .
COPY --from=builder /app/target/release/migration ./migration

# Copy templates and static files
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static

# Expose the application port
EXPOSE 3737

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3737/health || exit 1

# Create startup script to run migrations then start app
RUN echo '#!/bin/sh\n\
echo "Running database migrations..."\n\
./migration\n\
echo "Starting application..."\n\
exec ./actix-web-template' > /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

# Run the startup script
CMD ["/app/entrypoint.sh"]
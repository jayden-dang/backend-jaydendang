FROM rust:1.87.0-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/*

# Copy only Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy all Cargo.toml files from crates
COPY crates/core/jd_core/Cargo.toml ./crates/core/jd_core/
COPY crates/gateways/api_gateway/Cargo.toml ./crates/gateways/api_gateway/
COPY crates/gateways/web_server/Cargo.toml ./crates/gateways/web_server/
COPY crates/infrastructure/jd_infra/Cargo.toml ./crates/infrastructure/jd_infra/
COPY crates/infrastructure/jd_messaging/Cargo.toml ./crates/infrastructure/jd_messaging/
COPY crates/infrastructure/jd_storage/Cargo.toml ./crates/infrastructure/jd_storage/
COPY crates/infrastructure/jd_tracing/Cargo.toml ./crates/infrastructure/jd_tracing/
COPY crates/processors/analytics_processor/Cargo.toml ./crates/processors/analytics_processor/
COPY crates/processors/notification_processor/Cargo.toml ./crates/processors/notification_processor/
COPY crates/services/user_service/Cargo.toml ./crates/services/user_service/
COPY crates/shared/jd_contracts/Cargo.toml ./crates/shared/jd_contracts/
COPY crates/shared/jd_deencode/Cargo.toml ./crates/shared/jd_deencode/
COPY crates/shared/jd_domain/Cargo.toml ./crates/shared/jd_domain/
COPY crates/shared/jd_rpc_core/Cargo.toml ./crates/shared/jd_rpc_core/
COPY crates/shared/jd_streams/Cargo.toml ./crates/shared/jd_streams/
COPY crates/shared/jd_utils/Cargo.toml ./crates/shared/jd_utils/

# Build dependencies first
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    cargo build --release

# Now copy the actual source code
COPY . .

# Build the application
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release && \
    find target/release -maxdepth 1 -type f -executable -exec cp {} ./app \;

# Redis stage for development
FROM redis:7.2-alpine AS redis

# Production stage
FROM amazonlinux:2023 AS deploy

# Install runtime dependencies
RUN set -eux; \
    dnf update -y && dnf install -y \
    ca-certificates \
    curl-minimal \
    bind-utils \
    iputils \
    iproute \
    htop \
    jq \
    shadow-utils \
    && dnf clean all \
    && rm -rf /var/cache/dnf/*

# Copy Redis binary for development
COPY --from=redis /usr/local/bin/redis-server /usr/local/bin/
COPY --from=redis /usr/local/bin/redis-cli /usr/local/bin/

# Create Redis directory
RUN mkdir -p /var/lib/redis && \
    chown -R appuser:appuser /var/lib/redis

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /deploy

# Copy binary from builder
COPY --from=builder /app/app ./

# Set proper permissions
RUN chown -R appuser:appuser /deploy

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV DATABASE_URL=postgresql://jayden:postgres@localhost:5432/jaydenblog
ENV REDIS_URL=redis://localhost:6379

# Add security headers
ENV RUSTFLAGS="-C target-feature=+crt-static"

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./app"]

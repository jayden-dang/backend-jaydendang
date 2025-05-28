FROM rust:1.87.0-bullseye AS builder

# Add metadata labels
LABEL maintainer="Jayden Dang <jayden.dangvu@gmail.com>"
LABEL version="0.0.1"
LABEL description="Web server for Jayden Blog"

# Add build arguments
ARG APP_USER=appuser
ARG APP_UID=1000

WORKDIR /app

# Install build dependencies and security tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-audit

# Configure Rust toolchain
RUN rustup default stable && \
    rustup update

# Copy dependency files first for better caching
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

# Create dummy lib.rs files for library crates and main.rs for web_server
RUN mkdir -p crates/core/jd_core/src && \
    echo "pub fn dummy() {}" > crates/core/jd_core/src/lib.rs && \
    mkdir -p crates/gateways/api_gateway/src && \
    echo "pub fn dummy() {}" > crates/gateways/api_gateway/src/lib.rs && \
    mkdir -p crates/gateways/web_server/src && \
    echo "fn main() {}" > crates/gateways/web_server/src/main.rs && \
    mkdir -p crates/infrastructure/jd_infra/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_infra/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_messaging/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_messaging/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_storage/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_storage/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_tracing/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_tracing/src/lib.rs && \
    mkdir -p crates/processors/analytics_processor/src && \
    echo "pub fn dummy() {}" > crates/processors/analytics_processor/src/lib.rs && \
    mkdir -p crates/processors/notification_processor/src && \
    echo "pub fn dummy() {}" > crates/processors/notification_processor/src/lib.rs && \
    mkdir -p crates/services/user_service/src && \
    echo "pub fn dummy() {}" > crates/services/user_service/src/lib.rs && \
    mkdir -p crates/shared/jd_contracts/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_contracts/src/lib.rs && \
    mkdir -p crates/shared/jd_deencode/src && \
    echo '#[proc_macro_derive(Dummy)] pub fn dummy(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }' > crates/shared/jd_deencode/src/lib.rs && \
    mkdir -p crates/shared/jd_domain/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_domain/src/lib.rs && \
    mkdir -p crates/shared/jd_rpc_core/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_rpc_core/src/lib.rs && \
    mkdir -p crates/shared/jd_streams/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_streams/src/lib.rs && \
    mkdir -p crates/shared/jd_utils/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_utils/src/lib.rs

# Build dependencies with caching
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    cargo build --release

# Run security audit with warnings allowed
RUN cargo audit --deny warnings || true

# Now copy the actual source code
COPY . .

# Build the application with optimizations and strip debug symbols
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release && \
    find target/release -maxdepth 1 -type f -executable -exec cp {} ./app \; && \
    strip ./app

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

# Create non-root user first
RUN useradd -m -u 1000 appuser

# Copy Redis binary for development
COPY --from=redis /usr/local/bin/redis-server /usr/local/bin/
COPY --from=redis /usr/local/bin/redis-cli /usr/local/bin/

# Create Redis directory and set permissions
RUN mkdir -p /var/lib/redis && \
    chown -R appuser:appuser /var/lib/redis

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
ENV RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-s"

# Add health check with more detailed configuration
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Add proper signal handling
STOPSIGNAL SIGTERM

# Add proper entrypoint
ENTRYPOINT ["./app"]
CMD []

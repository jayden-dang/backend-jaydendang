FROM --platform=linux/amd64 rust:1.87.0-bullseye AS builder

# Add metadata labels
LABEL maintainer="Jayden Dang <jayden.dangvu@gmail.com>"
LABEL version="0.0.1"
LABEL description="Web server for Jayden Blog"

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-audit

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
COPY crates/services/sui_service/Cargo.toml ./crates/services/sui_service/
COPY crates/shared/jd_contracts/Cargo.toml ./crates/shared/jd_contracts/
COPY crates/shared/jd_typedenum/Cargo.toml ./crates/shared/jd_typedenum/
COPY crates/shared/jd_domain/Cargo.toml ./crates/shared/jd_domain/
COPY crates/shared/jd_rpc_core/Cargo.toml ./crates/shared/jd_rpc_core/
COPY crates/shared/jd_streams/Cargo.toml ./crates/shared/jd_streams/
COPY crates/shared/jd_utils/Cargo.toml ./crates/shared/jd_utils/

# Create dummy source files for dependency caching
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
    mkdir -p crates/services/sui_service/src && \
    echo "pub fn dummy() {}" > crates/services/sui_service/src/lib.rs && \
    mkdir -p crates/shared/jd_contracts/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_contracts/src/lib.rs && \
    mkdir -p crates/shared/jd_typedenum/src && \
    echo '#[proc_macro_derive(Dummy)] pub fn dummy(_: proc_macro::TokenStream) -> proc_macro::TokenStream { proc_macro::TokenStream::new() }' > crates/shared/jd_typedenum/src/lib.rs && \
    mkdir -p crates/shared/jd_domain/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_domain/src/lib.rs && \
    mkdir -p crates/shared/jd_rpc_core/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_rpc_core/src/lib.rs && \
    mkdir -p crates/shared/jd_streams/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_streams/src/lib.rs && \
    mkdir -p crates/shared/jd_utils/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_utils/src/lib.rs

# Build dependencies
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release

# Copy actual source code
COPY . .

# Build the application (native AMD64)
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --workspace --release && \
    echo "=== Built binaries ===" && \
    ls -la target/release/ && \
    echo "=== Copying web_server binary ===" && \
    cp target/release/web_server ./app && \
    ls -la ./app && \
    strip ./app

# Runtime stage - Ubuntu for compatibility
FROM --platform=linux/amd64 ubuntu:22.04 AS deploy

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*Creates a default .env file in the image

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /deploy

# Copy binary from builder
COPY --from=builder /app/app ./app

# Create default .env file trong image
RUN echo 'DATABASE_URL=postgres://jayden:postgres@localhost:5432/jaydenblog' > .env && \
    echo 'REDIS_URL=redis://localhost:6379/' >> .env && \
    echo 'WEB_ADDR=0.0.0.0:8080' >> .env && \
    echo 'RUST_LOG=info' >> .env && \
    echo 'RUST_BACKTRACE=1' >> .env

# Set permissions
RUN chmod +x ./app && chown -R appuser:appuser /deploy

# Switch to non-root user
USER appuser

# Set environment variables (backup nếu .env không work)
ENV DATABASE_URL=postgres://jayden:postgres@localhost:5432/jaydenblog
ENV REDIS_URL=redis://localhost:6379/
ENV WEB_ADDR=0.0.0.0:8080
ENV HOST=0.0.0.0
ENV PORT=8080
ENV BIND=0.0.0.0:8080
ENV SERVER_ADDR=0.0.0.0:8080
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run application
ENTRYPOINT ["./app"]

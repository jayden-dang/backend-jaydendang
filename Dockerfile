FROM rust:1.87.0-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/*

# Copy all source code first
COPY . .

# Build with optimizations and caching
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    rustup install stable; \
    RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release; \
    find target/release -maxdepth 1 -type f -executable -exec cp {} ./app \;

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
ENV TZ=Asia/Ho_Chi_Minh

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./app"]

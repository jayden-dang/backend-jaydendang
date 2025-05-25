FROM rust:1.70.0-slim-bullseye as builder

WORKDIR /app

COPY ./rust-toolchain ./
COPY ./Cargo.lock ./
COPY ./Cargo.toml ./
COPY ./.env ./.env
COPY ./crates/core/jd_core ./crates/jd_core
COPY ./crates/gateways/api_gateway ./crates/gateways/api_gateway
COPY ./crates/gateways/web_server ./crates/gateways/web_server
COPY ./crates/infrastructure/jd_infra ./crates/infrastructure/jd_infra
COPY ./crates/infrastructure/jd_messaging ./crates/infrastructure/jd_messaging
COPY ./crates/infrastructure/jd_storage ./crates/infrastructure/jd_storage
COPY ./crates/infrastructure/jd_tracing ./crates/infrastructure/jd_tracing
COPY ./crates/services/user_service ./crates/services/user_service
COPY ./crates/shared/jd_contracts ./crates/shared/jd_contracts
COPY ./crates/shared/jd_domain ./crates/shared/jd_domain
COPY ./crates/shared/jd_rpc_core ./crates/shared/jd_rpc_core
COPY ./crates/shared/jd_streams ./crates/shared/jd_streams
COPY ./crates/shared/jd_utils ./crates/shared/jd_utils
COPY ./crates/processors/analytics_processor ./crates/processors/analytics_processor
COPY ./crates/processors/notification_processor ./crates/processors/notification_processor

# on rebuilds, we explicitly cache our rust build dependencies to speed things up
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    rustup install stable; \
    cargo build --workspace --release; \
    objcopy --compress-debug-sections target/release/eamon_bin ./eamon

# stage two - we'll utilize a second container to run our built binary from our first container - slim containers!
FROM debian:11.3-slim as deploy

RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
    apt update; \
    apt install --yes --no-install-recommends bind9-dnsutils iputils-ping iproute2 curl ca-certificates htop; \
    apt clean autoclean; \
    apt autoremove --yes; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/;

WORKDIR /deploy

COPY --from=builder /app/jayden ./

CMD ["./jayden"]

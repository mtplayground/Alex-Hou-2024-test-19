FROM rust:1.90-bookworm AS builder

ARG CARGO_LEPTOS_VERSION=0.3.6

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        curl \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked cargo-leptos --version ${CARGO_LEPTOS_VERSION}

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo leptos build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --home-dir /app --create-home app

WORKDIR /app

COPY --from=builder /app/Cargo.toml ./Cargo.toml
COPY --from=builder /app/target/site ./target/site
COPY --from=builder /app/target/release/alex-hou-2024-test-19 /usr/local/bin/alex-hou-2024-test-19

ENV LEPTOS_SITE_ADDR=0.0.0.0:8080
ENV RUST_LOG=info,alex_hou_2024_test_19=debug

EXPOSE 8080

USER app

CMD ["alex-hou-2024-test-19"]

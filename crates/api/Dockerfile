FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libsqlite3-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p stilltrue-api

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/stilltrue-api /usr/local/bin/stilltrue-api
RUN mkdir -p /app/data

ENV PORT=8788 \
    DATABASE_URL=sqlite:///app/data/stilltrue.db?mode=rwc \
    RUST_LOG=stilltrue_api=info,tower_http=info

EXPOSE 8788
CMD ["stilltrue-api"]

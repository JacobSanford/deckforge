FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build application
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/deckforge /usr/local/bin/deckforge
COPY config.toml /etc/deckforge/config.toml

EXPOSE 3000
VOLUME /data

ENV DECKFORGE_DATA_DIR=/data
ENV DECKFORGE_LISTEN_ADDR=0.0.0.0:3000

CMD ["deckforge", "--config", "/etc/deckforge/config.toml", "start-server"]

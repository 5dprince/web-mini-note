FROM rust:1-slim AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools pkg-config ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    rustup set profile minimal && rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl || true

COPY src ./src
COPY public ./public
COPY styles.css copy.js markdown.js history.js script.js clippy.svg favicon.ico ./

ENV RUSTFLAGS="-C opt-level=z -C strip=symbols"
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN mkdir -p /app/_tmp

FROM alpine:3.20 AS certs
RUN apk add --no-cache ca-certificates

FROM scratch AS runtime
WORKDIR /app

ENV PORT=8080 \
    SAVE_PATH=_tmp \
    FILE_LIMIT=100000 \
    SINGLE_FILE_SIZE_LIMIT=10240 \
    STATIC_ROOT=.

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/web-note-rust /app/web-note-rust
COPY public ./public
COPY styles.css copy.js markdown.js history.js script.js clippy.svg favicon.ico ./
COPY --from=builder --chown=10001:10001 /app/_tmp/ /app/_tmp/

USER 10001:10001
EXPOSE 8080
VOLUME ["/app/_tmp"]
CMD ["/app/web-note-rust"]


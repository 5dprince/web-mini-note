# syntax=docker/dockerfile:1

FROM rust:1-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && cargo build --release || true
COPY src ./src
COPY public ./public
COPY styles.css copy.js markdown.js history.js clippy.svg favicon.ico ./
RUN cargo build --release

FROM debian:stable-slim AS runtime
WORKDIR /app
ENV PORT=8080 \
    SAVE_PATH=_tmp \
    FILE_LIMIT=100000 \
    SINGLE_FILE_SIZE_LIMIT=10240 \
    STATIC_ROOT=.
RUN set -eux; \
    apt-get update; apt-get install -y --no-install-recommends ca-certificates; \
    rm -rf /var/lib/apt/lists/*; \
    useradd -m -u 10001 appuser
COPY --from=builder /app/target/release/web-note-rust /app/web-note-rust
COPY public ./public
COPY styles.css copy.js markdown.js history.js clippy.svg favicon.ico ./
RUN mkdir -p /app/_tmp && chown -R appuser:appuser /app
USER appuser
EXPOSE 8080
VOLUME ["/app/_tmp"]
CMD ["/app/web-note-rust"]



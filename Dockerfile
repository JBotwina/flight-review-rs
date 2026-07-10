FROM rust:1.94-bookworm AS builder

WORKDIR /build

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build a portable release image. SQLite remains the crate default; these
# additional features make the same image work with Postgres and S3-compatible
# providers such as Railway Buckets.
ARG SERVER_FEATURES=postgres,s3
RUN cargo build --release \
      -p flight-review-server --features "$SERVER_FEATURES" \
      -p flight-review --bins \
    && mkdir -p /out \
    && cp /build/target/release/flight-review-server /out/ \
    && cp /build/target/release/ulog-convert /out/

FROM node:22-bookworm-slim AS frontend

WORKDIR /build/frontend

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci --include=optional --no-audit --no-fund

COPY frontend/ ./
ARG PUBLIC_MAPBOX_TOKEN=""
ENV PUBLIC_MAPBOX_TOKEN=${PUBLIC_MAPBOX_TOKEN}
RUN npx svelte-kit sync && npm run build

# Runtime image — minimal, no Rust toolchain
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl jq \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 flightreview \
    && useradd --uid 10001 --gid flightreview --no-create-home --shell /usr/sbin/nologin flightreview

COPY --from=builder /out/flight-review-server /usr/local/bin/
COPY --from=builder /out/ulog-convert /usr/local/bin/
COPY --from=frontend /build/frontend/build /usr/share/flight-review/frontend
COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint
COPY scripts/seed-logs.sh /usr/local/bin/seed-logs
COPY seed/logs.json /usr/share/flight-review/seed/logs.json

# Default data directory
RUN mkdir -p /data/files \
    && chmod 0755 /usr/local/bin/docker-entrypoint /usr/local/bin/seed-logs \
    && chown -R flightreview:flightreview /data

USER flightreview
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl --fail --silent "http://localhost:${PORT:-8080}/health" || exit 1

ENTRYPOINT ["docker-entrypoint"]
CMD ["flight-review-server", "serve", "--frontend-dir", "/usr/share/flight-review/frontend"]

# Araf backend multi-stage Docker build.
#
# Stage 1: Build both BFF binaries
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY backend .

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bin tenant-bff --bin operator-bff && \
    cp target/release/tenant-bff /usr/local/bin/tenant-bff && \
    cp target/release/operator-bff /usr/local/bin/operator-bff

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/tenant-bff /usr/local/bin/tenant-bff
COPY --from=builder /usr/local/bin/operator-bff /usr/local/bin/operator-bff

EXPOSE 8080 8081

CMD ["tenant-bff"]

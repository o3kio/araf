#
# Araf backend multi-stage Docker build.
#
# Build context must be the backend/ directory (set in docker-compose.yml).
# See docker-compose.yml for how this is invoked.
#

# Stage 1: Build both BFF binaries
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy the entire backend workspace (Cargo.toml, Cargo.lock, all crates)
COPY . .

# Build both binaries
RUN cargo build --release --bin tenant-bff --bin operator-bff && \
    cp target/release/tenant-bff /usr/local/bin/tenant-bff && \
    cp target/release/operator-bff /usr/local/bin/operator-bff

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates wget && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/tenant-bff /usr/local/bin/tenant-bff
COPY --from=builder /usr/local/bin/operator-bff /usr/local/bin/operator-bff

EXPOSE 8080 8081

CMD ["tenant-bff"]

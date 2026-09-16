#
# Araf backend multi-stage Docker build.
#
# Build context must be the backend/ directory (set in docker-compose.yml).
# See docker-compose.yml for how this is invoked.
#

# Stage 1: Build both BFF binaries
FROM rust:1.95-slim-bookworm@sha256:d7482085ff5b415f84dba5647ae71606650bdef00db7aeb69f4b3d170c3e4082 AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy the entire backend workspace (Cargo.toml, Cargo.lock, all crates)
COPY . .

# Build both binaries
RUN cargo build --release --bin tenant-bff --bin operator-bff && \
    cp target/release/tenant-bff /usr/local/bin/tenant-bff && \
    cp target/release/operator-bff /usr/local/bin/operator-bff

# Stage 2: Minimal runtime image. Distroless keeps the OpenSSL runtime and CA
# bundle required by reqwest while avoiding a package manager and shell in the
# production process image. The debug variant contributes only the pinned
# BusyBox health probe used by Compose; the BFF still runs as non-root.
FROM gcr.io/distroless/cc-debian13:debug-nonroot@sha256:f525a9a37aed3e8a848f46cfe055999782d66ed797e9e2886928c8caaaa4fc52
ARG ARAF_VERSION=0.0.0-dev
ARG ARAF_GIT_SHA=unknown
LABEL org.opencontainers.image.title="Araf BFF" \
      org.opencontainers.image.version="$ARAF_VERSION" \
      org.opencontainers.image.revision="$ARAF_GIT_SHA" \
      org.opencontainers.image.source="https://github.com/o3kio/araf"

COPY --from=builder /usr/local/bin/tenant-bff /usr/local/bin/tenant-bff
COPY --from=builder /usr/local/bin/operator-bff /usr/local/bin/operator-bff
ENV ARAF_VERSION=$ARAF_VERSION ARAF_GIT_SHA=$ARAF_GIT_SHA

# Seed the ownership of the durable state mount. Docker copies the image
# directory metadata into a newly-created named volume, allowing the
# non-root BFF (UID 65532) to initialize sessions and the compatibility
# journal without a privileged post-start repair.
RUN mkdir -p /var/lib/araf && chown 65532:65532 /var/lib/araf

# Clear the debug distroless shell entrypoint so Compose commands execute the
# selected BFF binary directly.
ENTRYPOINT []

EXPOSE 8080 8081

CMD ["tenant-bff"]

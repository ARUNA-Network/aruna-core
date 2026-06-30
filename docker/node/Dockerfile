# =============================================================================
# ARUNA Network — Node Images
#
# Images produced:
#   ghcr.io/aruna-network/node-builder  — build environment + compiled binary
#   ghcr.io/aruna-network/node-runtime  — minimal production image (shipped)
#
# Build targets:
#   builder  → docker build -f docker/node/Dockerfile --target builder ...
#   runtime  → docker build -f docker/node/Dockerfile --target runtime ...
#              (default target, no --target flag needed)
#
# Both targets derive from the ODE image, ensuring identical toolchain.
# The runtime image contains ONLY the binary, config, and runtime libraries.
# It has no compiler, no Rust, no build toolchain.
#
# ADR reference: docs/adr/ (Infrastructure / Docker)
# =============================================================================

# ── Stage 1: builder ─────────────────────────────────────────────────────────
# Uses the published ODE image as base — guarantees identical toolchain to CI
# and DevContainer. Override ODE_IMAGE in CI for reproducible builds.
ARG ODE_IMAGE=aruna-ode:local
FROM ${ODE_IMAGE} AS builder

WORKDIR /usr/src/aruna
COPY . .

# Build the node binary in release mode
RUN cargo build --release -p aruna-node

# Verify the binary was produced
RUN test -f target/release/aruna-node \
    && echo "✓ aruna-node binary produced ($(du -sh target/release/aruna-node | cut -f1))"


# ── Stage 2: runtime ─────────────────────────────────────────────────────────
# Minimal production image. Contains ONLY what is needed to run the node:
#   - aruna-node binary
#   - config directory
#   - pinned runtime libraries
#   - TLS certificates
# No compiler, no Rust, no build toolchain.
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Pinned runtime libraries only — audited 2026-06-30 against ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
    "libssl-dev=3.0.13-0ubuntu3.11" \
    ca-certificates \
    "curl=8.5.0-2ubuntu10.9" \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the binary and config from the builder stage
COPY --from=builder /usr/src/aruna/target/release/aruna-node /app/aruna-node
COPY --from=builder /usr/src/aruna/config /app/config

# Expose standard ARUNA Network ports (9000: P2P, 8080: RPC)
EXPOSE 9000
EXPOSE 8080

ENTRYPOINT ["/app/aruna-node"]
CMD ["--p2p-port", "9000", "--rpc-port", "8080"]

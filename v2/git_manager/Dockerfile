# ============================================================
# Git Multi-Account Manager — Multi-Stage Docker Build
# Architecture: Zig native layer → Rust workspace → runtime
# ============================================================

# ──────────────────────────────────────────────────────────────
# Stage 1: Zig native library (libgm_native.a)
# ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS zig-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    xz-utils \
    libsecret-1-dev \
    libglib2.0-dev \
    pkg-config \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

ARG ZIG_VERSION=0.14.0
ARG ZIG_ARCH=x86_64-linux
RUN curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz" \
    | tar -xJ -C /usr/local --strip-components=1 \
    && ln -s /usr/local/zig /usr/local/bin/zig

WORKDIR /build
COPY zig_native/ zig_native/
RUN cd zig_native && zig build -Doptimize=ReleaseSafe

# ──────────────────────────────────────────────────────────────
# Stage 2: Rust workspace (release binaries)
# ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    pkg-config \
    libsecret-1-dev \
    libglib2.0-dev \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY --from=zig-builder /build/zig_native/zig-out/ zig_native/zig-out/
COPY Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml deny.toml ./
COPY crates/ crates/
COPY apps/ apps/
COPY zig_native/build.zig zig_native/build.zig
COPY zig_native/src/ zig_native/src/
COPY zig_native/include/ zig_native/include/

ARG CARGO_PROFILE=release
RUN cargo build --workspace --profile ${CARGO_PROFILE} --bin git-manager-web

# ──────────────────────────────────────────────────────────────
# Stage 3: Production runtime image
# ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsecret-1-0 \
    ca-certificates \
    git \
    openssh-client \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && addgroup --system git-manager \
    && adduser --system --ingroup git-manager --home /var/lib/git-manager git-manager

COPY --from=rust-builder /build/target/release/git-manager-web /usr/bin/git-manager-web

USER git-manager
WORKDIR /var/lib/git-manager

EXPOSE 5008

ENV GIT_MANAGER_WEB_ADDR=0.0.0.0:5008
ENV RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:5008/health || exit 1

ENTRYPOINT ["/usr/bin/git-manager-web"]

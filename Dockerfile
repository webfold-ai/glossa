# syntax=docker/dockerfile:1.7
#
# glossa runtime image
# =====================
#
# Multi-stage musl/Alpine. glossa is a pure-Rust CLI for cross-language
# i18n catalogues — no native C deps beyond ring (TLS) and the musl
# libc. Alpine is the right base.
#
# Build:
#   docker build -t glossa:latest --build-arg BINARY=glossa .
#
# Cross-compile: TARGET=x86_64-unknown-linux-musl

ARG RUST_BUILDER_IMAGE=rust:1.88-alpine
ARG RUNTIME_IMAGE=alpine:3.20
ARG BINARY=glossa
ARG TARGET=x86_64-unknown-linux-musl

# ----------------------------------------------------------------------------
# Stage 1: builder
# ----------------------------------------------------------------------------
FROM ${RUST_BUILDER_IMAGE} AS builder

# glossa has no native C deps beyond TLS. `pkgconfig` is for the
# `ring` musl build to find the static openssl.
RUN apk add --no-cache \
        build-base \
        pkgconfig \
        openssl-dev \
        openssl-libs-static \
        git \
        ca-certificates \
    && update-ca-certificates

WORKDIR /build

ENV CARGO_TARGET_DIR=/build/target
ENV CC_x86_64_unknown_linux_musl=musl-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

# Pre-cache deps.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release --target ${TARGET} --locked && \
    rm -rf src target/${TARGET}/release/deps/glossa*

# Real source (incl. the `i18n/` and `examples/` trees that are
# part of glossa).
COPY . .

RUN cargo build --release --target ${TARGET} --locked --bin ${BINARY} && \
    cp target/${TARGET}/release/${BINARY} /usr/local/bin/${BINARY} && \
    strip /usr/local/bin/${BINARY}

# CycloneDX SBOM for the binary.
RUN cargo install --locked cargo-cyclonedx --no-default-features || true
RUN if command -v cargo-cyclonedx >/dev/null; then \
        cargo cyclonedx -p ${BINARY} --format json \
            --target ${TARGET} \
            --override-filename "/usr/local/share/${BINARY}.cdx.json" || true; \
    fi

# ----------------------------------------------------------------------------
# Stage 2: runtime
# ----------------------------------------------------------------------------
FROM ${RUNTIME_IMAGE} AS runtime

RUN apk add --no-cache \
        ca-certificates \
        tzdata \
        libgcc \
    && update-ca-certificates

COPY --from=builder /usr/local/bin/${BINARY} /usr/local/bin/
COPY --from=builder /usr/local/share/*.cdx.json /usr/local/share/ 2>/dev/null || true

RUN addgroup -g 65532 glossa 2>/dev/null || true; \
    adduser -u 65532 -G glossa -s /sbin/nologin -D glossa 2>/dev/null || true
USER 65532:65532

ENTRYPOINT ["/usr/local/bin/glossa"]
CMD ["--help"]

# ----------------------------------------------------------------------------
# OCI labels
# ----------------------------------------------------------------------------
ARG VCS_REF=unknown
ARG BUILD_DATE=unknown
ARG IMAGE_VERSION=dev

LABEL org.opencontainers.image.title="${BINARY}" \
      org.opencontainers.image.description="glossa — cross-language i18n catalogue CLI" \
      org.opencontainers.image.source="https://github.com/webfold-ai/glossa" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.version="${IMAGE_VERSION}" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.vendor="webfold-ai" \
      org.opencontainers.image.base.image="alpine:3.20"

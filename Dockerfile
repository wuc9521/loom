FROM rust:bookworm AS builder

ARG RUST_TOOLCHAIN=nightly-2025-03-13

RUN rustup toolchain install ${RUST_TOOLCHAIN} --profile minimal --component cargo --component rustc --component rust-std \
    && rustup default ${RUST_TOOLCHAIN}

WORKDIR /workspace

# Build context is the loom checkout (with the loom-bv submodule initialized).
COPY Cargo.toml Cargo.lock build.rs README.md /workspace/loom/
COPY src /workspace/loom/src
COPY loom-bv/Cargo.toml loom-bv/build.rs loom-bv/README.md loom-bv/rust-toolchain.toml /workspace/loom/loom-bv/
COPY loom-bv/src /workspace/loom/loom-bv/src

WORKDIR /workspace/loom
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends cvc5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/loom/target/release/loom /usr/local/bin/loom
COPY benchmark /app/benchmark
COPY README.md /app/README.md

ENTRYPOINT ["/usr/local/bin/loom"]
CMD ["--help"]

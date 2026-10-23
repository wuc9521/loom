FROM rust:bookworm AS builder

ARG RUST_TOOLCHAIN=nightly-2025-03-13

RUN rustup toolchain install ${RUST_TOOLCHAIN} --profile minimal --component cargo --component rustc --component rust-std \
    && rustup default ${RUST_TOOLCHAIN}

WORKDIR /workspace

# `loom` depends on the sibling dryadsynth-bv path crate, so the Docker build
# context contains both projects. sygus-parser is now resolved from crates.io.
COPY loom/Cargo.toml loom/Cargo.lock loom/build.rs loom/README.md /workspace/loom/
COPY loom/src /workspace/loom/src
COPY dryadsynth-bv/Cargo.toml dryadsynth-bv/Cargo.lock dryadsynth-bv/build.rs dryadsynth-bv/README.md dryadsynth-bv/rust-toolchain.toml /workspace/dryadsynth-bv/
COPY dryadsynth-bv/src /workspace/dryadsynth-bv/src

WORKDIR /workspace/loom
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends cvc5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/loom/target/release/loom /usr/local/bin/loom
COPY loom/benchmark /app/benchmark
COPY loom/README.md /app/README.md

ENTRYPOINT ["/usr/local/bin/loom"]
CMD ["--help"]

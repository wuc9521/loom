FROM rust:bookworm AS builder

ARG RUST_TOOLCHAIN=nightly-2025-03-13

RUN rustup toolchain install ${RUST_TOOLCHAIN} --profile minimal --component cargo --component rustc --component rust-std \
    && rustup default ${RUST_TOOLCHAIN}

WORKDIR /workspace

# `delis` depends on sibling path crates, so the Docker build context must
# contain `delis`, `sygus-parser-rs`, and `dryadsynth-bv`.
COPY delis/Cargo.toml delis/Cargo.lock delis/build.rs delis/README.md /workspace/delis/
COPY delis/src /workspace/delis/src
COPY sygus-parser-rs/Cargo.toml /workspace/sygus-parser-rs/
COPY sygus-parser-rs/src /workspace/sygus-parser-rs/src
COPY dryadsynth-bv/Cargo.toml dryadsynth-bv/Cargo.lock dryadsynth-bv/build.rs dryadsynth-bv/README.md dryadsynth-bv/rust-toolchain.toml /workspace/dryadsynth-bv/
COPY dryadsynth-bv/src /workspace/dryadsynth-bv/src

WORKDIR /workspace/delis
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends cvc5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/delis/target/release/delis /usr/local/bin/delis
COPY --from=builder /workspace/delis/target/release/delis-bench /usr/local/bin/delis-bench
COPY delis/benchmark /app/benchmark
COPY delis/benchmarks.toml /app/benchmarks.toml
COPY delis/README.md /app/README.md

ENTRYPOINT ["/usr/local/bin/delis"]
CMD ["--help"]

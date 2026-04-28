FROM rust:1.81-bookworm AS builder

WORKDIR /workspace

# `delis` depends on the sibling path crate `../sygus-parser-rs`,
# so the Docker build context must be the parent directory that contains both.
COPY delis/Cargo.toml delis/Cargo.lock delis/build.rs delis/README.md /workspace/delis/
COPY delis/src /workspace/delis/src
COPY sygus-parser-rs /workspace/sygus-parser-rs

WORKDIR /workspace/delis
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends cvc5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/delis/target/release/delis /usr/local/bin/delis
COPY delis/benchmark /app/benchmark
COPY delis/README.md /app/README.md

ENTRYPOINT ["/usr/local/bin/delis"]
CMD ["--help"]

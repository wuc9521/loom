# Loom

Loom learns logic formulas from finite models.

Today the default solver path is:

- build one semantic program
- lower it once to SyGuS BV
- race Loom's internal closure backend against the in-process `dryadsynth-bv` backend
- print the first successful result

`cvc5` is optional. Use it only when you explicitly select an external solver
with `--solver cvc5` or when you want to consume `--dump-sygus` output
yourself.

## Requirements

- Rust toolchain `nightly-2025-03-13` (see [`rust-toolchain.toml`](rust-toolchain.toml))
- sibling checkouts at `../sygus-parser-rs` and `../dryadsynth-bv`
- the `benchmark/` git submodule
- optional: `cvc5` for external SyGuS solving

Expected checkout layout:

```text
parent/
├── loom/
├── dryadsynth-bv/
└── sygus-parser-rs/
```

## Build

```bash
git submodule update --init --recursive
cargo build --locked
cargo run -- --help
```

Install the binary locally with:

```bash
cargo install --path . --locked
```

## Usage

```text
Loom: Learning logic formulas from finite models

Usage: loom [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the input file (if not using --in)

Options:
  -i, --in                     Read input from stdin instead of file
  -v, --verbose...             Increase log verbosity (use `-v` for INFO, `-vv` for DEBUG)
  -O, --optimize[=<OPTIMIZE>]  Enable SyGuS optimization passes (use -O=false or --optimize=false to disable) [default: true] [possible values: true, false]
      --coi[=<COI>]            Enable semantic candidate deduplication and relevance/circuit pruning in dryadsynth-bv; --coi=false disables both [default: true] [possible values: true, false]
  -B, --bisim[=<BISIM>]        Enable semantic bisimulation quotienting; use --bisim=false to skip it entirely [default: true] [possible values: true, false]
      --enum                   Use only Loom's internal enumerative backend (no SyGuS solver backend)
  -d, --dump-sygus             Translate to SyGuS BV and print to stdout
      --sygus-stats            Translate to SyGuS BV, but print only machine-readable stats: `SYGUS_STATS<TAB>ms<TAB>bytes<TAB>lines`
      --dump-chc               Translate to CHC (HORN) and print to stdout
      --cegis[=<N>]            Use counterexample-guided subset solving for the external SyGuS backend, seeding with up to N constraints. Off by default; pass `--cegis` to enable with the default seed size, or `--cegis=N` to set the seed [default: 0]
      --solver <PATH>          Use an explicit external SyGuS solver executable instead of the in-process dryadsynth-bv backend
  -h, --help                   Print help
  -V, --version                Print version
```

## Examples

Solve a small CFG benchmark with the default backend race:

```bash
$ cargo run -- benchmark/CFG/anbn-1.sl
(
(define-fun g () CFG (cfg.top.S (cfg.++ a b) end))
)
```

Emit only SyGuS encoding stats:

```bash
$ cargo run -- --sygus-stats benchmark/CFG/anbn-1.sl
SYGUS_STATS	33	9952	17
```

Dump the generated SyGuS problem:

```bash
cargo run -- --dump-sygus benchmark/CFG/anbn-1.sl > /tmp/anbn-1.sygus
```

Try an explicit external solver instead of the in-process `dryadsynth-bv`
backend:

```bash
cargo run -- --solver cvc5 benchmark/CFG/anbn-1.sl
```

External-solver behavior is benchmark-dependent. `--cegis` applies only to the
external SyGuS backend:

```bash
cargo run -- --solver cvc5 --cegis benchmark/CFG/anbn-1.sl
```

## Docker

`loom` depends on the sibling path crates `../sygus-parser-rs` and
`../dryadsynth-bv`, so the Docker build context must be the parent directory
that contains all three repositories.

```bash
cd /path/to/parent/of/loom-dryadsynth-bv-and-sygus-parser-rs
docker build -f loom/Dockerfile -t loom:latest .
docker run --rm loom:latest --help
docker run --rm -v "$PWD/loom:/work" -w /work loom:latest /work/benchmark/CFG/anbn-1.sl
```

The runtime image installs `cvc5`, and the repository `Makefile` can also
export the image as `dist/loom.tar.gz` via `make docker`.

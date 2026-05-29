# Delis

Install dependency `cvc5` for the default solver setup.
By default, Delis races the compiled SyGuS problems with `cvc5`.
The first decisive backend result ends the run.
When the semantic bit-vector encoding length is at least `--enum-trigger`,
Delis runs SyGuS with `cvc5 --lang=sygus2 --sygus-enum=fast`.
The default trigger is `512`.

Usage

```
$ cargo install --path . --locked 
$ delis --help                                                    
Delis: Learning logic formulas from finite models

Usage: delis [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the input file (if not using --in)

Options:
  -i, --in                           Read input from stdin instead of file
  -v, --verbose                      Enable verbose output
  -O, --optimize[=<OPTIMIZE>]        Enable SyGuS optimization passes (use -O=false or --optimize=false to disable) [default: true] [possible values: true, false]
  -d, --dump-sygus                   Translate to SyGuS BV and print to stdout
      --dump-chc                     Translate to CHC (HORN) and print to stdout
      --dump-chc-witness             Translate to Z3 fixedpoint CHC witness query and print to stdout
      --chc-witness                  Solve using only the Z3 fixedpoint CHC witness path
      --enum-trigger <ENUM_TRIGGER>  Use `cvc5 --sygus-enum=fast` when the semantic BV encoding length is at least this value [default: 512]
      --cegis[=<N>]                  Use counterexample-guided subset solving for the external SyGuS backend; default seed is 6, use --cegis=0 to disable
  -h, --help                         Print help
  -V, --version                      Print version
```

Demo

```bash
time delis ./benchmark/fok/folsep-pldi20/alternating_bit_protocol-abp_ack_msg.sl 
(
(define-fun phi () Formula (fol.forall.x0.abp.ack_msg_t (fol.or (fol.eq.abp.ack_msg_t fol.var.x0.abp.ack_msg_t fol.var.x1.abp.ack_msg_t) (fol.forall.x1.abp.ack_msg_t (fol.atom.abp.ack_msg.le fol.var.x0.abp.ack_msg_t fol.var.x1.abp.ack_msg_t)))))
)
delis ./benchmark/fok/folsep-pldi20/alternating_bit_protocol-abp_ack_msg.sl  0.14s user 0.02s system 129% cpu 0.123 total
```

Docker

`delis` depends on the sibling path crates `../sygus-parser-rs` and
`../dryadsynth-bv`, so the Docker build context must be the parent directory
that contains all three repositories.

```bash
cd /path/to/parent/of/delis-sygus-parser-rs-and-dryadsynth-bv
docker build -f delis/Dockerfile -t delis:latest .
docker run --rm delis:latest --help
docker run --rm -v "$PWD/delis:/work" -w /work delis:latest ./benchmark/alpharegex/re/no20.sl --verbose
```

The runtime image installs `cvc5`, which is the default external solver used by
Delis.

Benchmark workflow

The benchmarking entrypoint is the standalone crate in
`bench-runner/`. This keeps benchmark orchestration independent from the
solver's nightly-only dependencies and makes the workflow resumable and
manifest-driven.

Build the runner:

```bash
cargo build --manifest-path bench-runner/Cargo.toml --release
```

Inspect the configured suites:

```bash
./bench-runner/target/release/delis-bench list benchmarks.toml
```

Run all pending jobs with automatic parallelism:

```bash
./bench-runner/target/release/delis-bench run benchmarks.toml
```

Resume is automatic. The runner stores append-only state in
`bench-results/runs/<run-id>/results.jsonl` and rewrites the top-level
`bench-results/<suite>.tsv` exports after each completed job.

Useful variants:

```bash
# Run a subset of suites.
./bench-runner/target/release/delis-bench run benchmarks.toml --suite reg-so --suite reg-deepregex

# Restrict to matching benchmark paths.
./bench-runner/target/release/delis-bench run benchmarks.toml --filter no10.sl

# Check the latest run state without executing new jobs.
./bench-runner/target/release/delis-bench status benchmarks.toml

# Force a fresh run lineage instead of resuming the latest compatible run.
./bench-runner/target/release/delis-bench run benchmarks.toml --fresh
```

Manifest notes:

- Top-level defaults live in `benchmarks.toml`.
- `[[suites]]` is the preferred name, but the runner still accepts the old
  `[[folders]]` entries for compatibility.
- Each suite can override `mode`, `delis`, `delis_args`, `sygus_solver`,
  `sygus_solver_args`, `solve_timeout_secs`, `sygus_timeout_secs`, and
  `capture_sygus_stats`.
- `mode = "external_sygus"` keeps Delis only as a SyGuS front-end and sends the
  generated problem to the configured external solver.
- `mode = "external_command"` runs an arbitrary wrapper command per benchmark
  file. Use `file_extensions`, `working_dir`, `external_command`, and
  `external_command_args` to describe the adapter.

The legacy shell scripts (`bench.sh`, `bench-cvc5.sh`, `bench-syslite.sh`,
`bench-samples2LTL.sh`, `bench-folseparators.sh`) are kept for now as
special-purpose helpers, but the intended Delis benchmarking path is the new
runner above.

External comparisons

The external-comparison manifest lives in `benchmarks-external.toml`.
Currently it covers:

- `reg-alpharegex-cvc5`, `reg-deepregex-cvc5`, `reg-so-cvc5`
- `fo-folseparators-artifact`

Set up comparison tools on a machine with:

```bash
scripts/setup-external-tools.sh --folseparators-artifact /path/to/560.tgz
```

Then run, for example:

```bash
./bench-runner/target/release/delis-bench run benchmarks-external.toml --suite reg-so-cvc5
./bench-runner/target/release/delis-bench run benchmarks-external.toml --suite fo-folseparators-artifact
```

The folseparators source corpus is now tracked under
`benchmark-upstream/FO/folseparators/`, recovered from the artifact Docker image.
A repo and benchmark inventory lives in `benchmark-upstream/EXTERNAL_TOOLS.md`.

Remote benchmarking

For a remote server, the simplest workflow is to build the Docker image locally
and export it as one tarball. This avoids any private-repository setup on the
server.

Local machine:

```bash
make docker
```

This writes `dist/delis.tar.gz`.

You can override the image name or output path if needed:

```bash
make docker IMAGE_TAG=my-delis:bench IMAGE_ARCHIVE=/tmp/my-delis-bench.tar.gz
```

By default the image is built for `linux/amd64`, which matches most remote
servers. You can override that if needed:

```bash
make docker IMAGE_PLATFORM=linux/arm64
```

Then copy the tarball to the server with `scp`, load it there with `docker
load`, and run the container as needed.

The image includes:

- `/usr/local/bin/delis`
- `/usr/local/bin/delis-bench`
- `/app/benchmark`
- `/app/benchmarks.toml`

Run the benchmark driver with the bundled manifest:

```bash
docker run --rm --entrypoint /usr/local/bin/delis-bench delis:bench /app/benchmarks.toml
```

`delis-bench` keeps the canonical run ledger under
`bench-results/runs/<run-id>/` and mirrors the latest per-suite TSV exports in
the top-level `bench-results/` directory for plotting scripts.

If you cloned this branch without populating the benchmark submodule, initialize
it before running the manifest:

```bash
git submodule update --init --recursive
```

# TODO

- [x] CEGIS: randomly choose n models and verify on the rest.
- [x] `--dump-sygus`.
- [?] Implement String / Flash.
- [x] `DryadSynth` on old benchmarks.
- [x] CTL.

```bash
(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/alpharegex/re/no10.sl                                  
WARN SyGuS file parsed, total cmds=27 dryadsynth-bv/src/main.rs:101
(re.++ (re.opt char.0) (re.++ (re.* (re.++ char.1 char.0)) (re.opt char.1)))
delis ./benchmark/alpharegex/re/no10.sl  324.44s user 2.05s system 199% cpu 2:43.46 total


(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/alpharegex/re/no10.sl --dump-sygus | cvc5 --lang=sygus2
^Ccvc5 interrupted by user.
delis ./benchmark/alpharegex/re/no10.sl --dump-sygus  0.06s user 0.01s system 78% cpu 0.091 total
cvc5 --lang=sygus2  240.98s user 2.10s system 99% cpu 4:03.54 total
```

```bash
(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/folsep-pldi20/fo1/hybrid_reliable_broadcast_cisa-rb_tp_correctness_impl4.sl --dump-sygus | dryadsynth-bv -in 
(
(define-fun phi () (_ BitVec 13) (fol.0.not (fol.0.and (fol.0.exists.x0.node (fol.ctx-x0-node.atom.rb.rcv_msg fol.ctx-x0-node.var.x0.node fol.ctx-x0-node.term.rb.tp_correctness.n0)) (fol.0.forall.x0.node (fol.ctx-x0-node.or (fol.ctx-x0-node.eq.node fol.ctx-x0-node.var.x0.node fol.ctx-x0-node.term.rb.tp_relay.n1) (fol.ctx-x0-node.atom.rb.rcv_msg fol.ctx-x0-node.var.x0.node fol.ctx-x0-node.term.rb.tp_correctness.n0))))))
)
delis  --dump-sygus  0.05s user 0.01s system 97% cpu 0.057 total
dryadsynth-bv -in  1.17s user 0.02s system 97% cpu 1.216 total


(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/folsep-pldi20/fo1/hybrid_reliable_broadcast_cisa-rb_tp_correctness_impl4.sl --dump-sygus | cvc5 --lang=sygus2
(
(define-fun phi () (_ BitVec 13) (fol.0.forall.x0.quorum_b (fol.ctx-x0-quorum_b.and (fol.ctx-x0-quorum_b.not (fol.ctx-x0-quorum_b.forall.x0.node (fol.ctx-x0-node.atom.rb.rcv_msg fol.ctx-x0-node.var.x0.node fol.ctx-x0-node.var.x0.node))) (fol.ctx-x0-quorum_b.or (fol.ctx-x0-quorum_b.atom.rb.rcv_msg fol.ctx-x0-quorum_b.term.rb.tp_correctness.n0 fol.ctx-x0-quorum_b.term.rb.tp_relay.n1) (fol.ctx-x0-quorum_b.atom.member_b fol.ctx-x0-quorum_b.term.rb.tp_relay.n1 fol.ctx-x0-quorum_b.var.x0.quorum_b)))))
)
delis  --dump-sygus  0.05s user 0.01s system 97% cpu 0.059 total
cvc5 --lang=sygus2  5.07s user 0.04s system 99% cpu 5.139 total
```

- [ ] String: simpler model representation / `charAt`
.

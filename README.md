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
      --cegis                        Use counterexample-guided subset solving for the external SyGuS backend
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

`delis` depends on the sibling path crate `../sygus-parser-rs`, so the Docker
build context must be the parent directory that contains both repositories.

```bash
cd /path/to/parent/of/delis-and-sygus-parser-rs
docker build -f delis/Dockerfile -t delis:latest .
docker run --rm delis:latest --help
docker run --rm -v "$PWD/delis:/work" -w /work delis:latest ./benchmark/alpharegex/re/no20.sl --verbose --cegis
```

The runtime image installs `cvc5`, which is the default external solver used by
Delis.

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

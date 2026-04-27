# Delis

Install dependency `cvc5` and `z3` for the default solver setup.
By default, Delis races the compiled CHC and SyGuS problems with `cvc5`.
The first decisive backend result ends the run.
When the semantic bit-vector encoding length is at least
`--enum-trigger`, Delis runs SyGuS with `cvc5 --lang=sygus2 --sygus-enum=fast`.
The default trigger is `512`.

Usage

```
$ ./target/release/delis --help                                                    
Parse and analyze your custom language files

Usage: delis [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the input file (if not using --in)

Options:
  -i, --in                      Read input from stdin instead of file
  -v, --verbose                 Enable verbose output
  -d, --dump-sygus              Translate to SyGuS BV and print to stdout
      --dump-chc                Translate to CHC (HORN) and print to stdout
      --enum-trigger <ENUM_TRIGGER>
                                  Use `cvc5 --sygus-enum=fast` when the semantic BV encoding length is at least this value [default: 512]
  -h, --help                    Print help
  -V, --version                 Print version
```

Demo

```bash
time delis ./benchmark/fok/folsep-pldi20/alternating_bit_protocol-abp_ack_msg.sl 
(
(define-fun phi () Formula (fol.forall.x0.abp.ack_msg_t (fol.or (fol.eq.abp.ack_msg_t fol.var.x0.abp.ack_msg_t fol.var.x1.abp.ack_msg_t) (fol.forall.x1.abp.ack_msg_t (fol.atom.abp.ack_msg.le fol.var.x0.abp.ack_msg_t fol.var.x1.abp.ack_msg_t)))))
)
delis ./benchmark/fok/folsep-pldi20/alternating_bit_protocol-abp_ack_msg.sl  0.14s user 0.02s system 129% cpu 0.123 total
```

# TODO

- [ ] CEGIS: randomly choose n models and verify on the rest.
- [ ] `--dump-sygus`.
- [ ] Implement String / Flash.
- [ ] `DryadSynth` on old benchmarks.
- [ ] CTL.

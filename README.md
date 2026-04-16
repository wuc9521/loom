# Delis

Install dependencies `cvc5` and `z3` for the default solver setup.

- `cvc5` should be built from source code due to issue [#12375](https://github.com/cvc5/cvc5/issues/12375)

By default, Delis runs CHC solving with `z3 -in` and SyGuS solving with `cvc5 --lang=sygus2`.
These defaults can be overridden when another solver accepts the generated input:

```bash
$ delis --chc-solver horn-solver --chc-solver-arg -in \
        --sygus-solver sygus-solver --sygus-solver-arg=--lang=sygus2 \
        benchmark/delis/re/no10_alternating_short.dl
```

`--chc-solver-arg` and `--sygus-solver-arg` may be repeated. Supplying a solver command or any solver argument replaces that solver's default argument list.

Usage

```
(base) *[main][~/Codes/Rust/delis]$ ./target/release/delis --help                                                    
Parse and analyze your custom language files

Usage: delis [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the input file (if not using --in)

Options:
  -i, --in                      Read input from stdin instead of file
  -v, --verbose                 Enable verbose output
  -d, --dump-sygus              Translate to SyGuS BV and print to stdout
      --dump-chc                Translate to CHC (HORN) and print to stdout
      --dump-semgus             Translate to SemGuS and print to stdout
      --linear                  Run CHC first and SyGuS second instead of running both solvers in parallel
      --chc-solver <COMMAND>    CHC solver command. Defaults to `z3`
      --chc-solver-arg <ARG>    CHC solver argument. May be repeated. Defaults to `-in` unless solver command or arguments are overridden
      --sygus-solver <COMMAND>  SyGuS solver command. Defaults to `cvc5`
      --sygus-solver-arg <ARG>  SyGuS solver argument. May be repeated. Defaults to `--lang=sygus2` unless solver command or arguments are overridden
  -h, --help                    Print help
  -V, --version                 Print version
```

Demo

```bash
$ cargo build --release
$ ./target/release/delis benchmark/delis/re/no10_alternating_short.dl 
(
(define-fun regex () RegLan (re.union a (re.union b (re.union (re.++ a b) (re.++ b (re.++ a (re.* b)))))))
)
```

# Delis

Install dependency `cvc5` for the default solver setup.

- `cvc5` should be built from source code due to issue [#12375](https://github.com/cvc5/cvc5/issues/12375)

By default, Delis races the compiled CHC and SyGuS problems with `cvc5`.
The first decisive backend result ends the run.
When the semantic bit-vector encoding length is at least
`--enum-trigger`, Delis runs SyGuS with `cvc5 --lang=sygus2 --sygus-enum=fast`.
The default trigger is `256`.

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
      --enum-trigger <ENUM_TRIGGER>
                                  Use `cvc5 --sygus-enum=fast` when the semantic BV encoding length is at least this value [default: 256]
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

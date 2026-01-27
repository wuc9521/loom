# Delis

Install dependencies `cvc5` and `z3`.

- `cvc5` should be built from source code due to issue [#12375](https://github.com/cvc5/cvc5/issues/12375)

Usage

```
(base) *[main][~/Codes/Rust/delis]$ ./target/release/delis --help                                                    
Parse and analyze your custom language files

Usage: delis [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the input file (if not using --in)

Options:
  -i, --in              Read input from stdin instead of file
  -v, --verbose         Enable verbose output
  -d, --dump-sygus      Translate to SyGuS BV and print to stdout
      --dump-chc        Translate to CHC (HORN) and print to stdout
      --dump-chc-sygus  Translate CHC to SyGuS BV and print to stdout
      --no-optim        Disable CHC to SyGuS rewrite optimization
  -a, --as-sygus        
  -h, --help            Print help
  -V, --version         Print version
```

Demo

```bash
$ cargo build --release
$ ./target/release/delis benchmark/delis/re/no10_alternating_short.dl 
(
(define-fun regex () RegLan (re.union a (re.union b (re.union (re.++ a b) (re.++ b (re.++ a (re.* b)))))))
)
```

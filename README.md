# Delis

Install dependencies `cvc5`.

Demo

```bash
cargo build --release

./target/release/delis --file benchmark/dl/re/no3_endwith_ab.sl --dump-sygus | cvc5 --lang=sygus2 
```

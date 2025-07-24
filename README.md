# Logic Learning Language Parser

Frontend of Logic Learning Problems

To run the project.

Install dependencies `cvc5`.

Some examples

```bash
# First Order Logic
cargo run -- --file ./exp/fol/finite-variable-2v-t12f34.sl --to-bv --dump . | cvc5 --lang=sygus2

# Propositional Modal Logic
cargo run -- --file ./exp/pml/meta-thm-t13f24.sl --to-bv --dump . | cvc5 --lang=sygus2

# Regular Expression
cargo run -- --input ./exp/re/1.sl 
```

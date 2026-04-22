```bash
(base) [main][~/Codes/Rust/delis]$ time delis ./benchmark/regel/cfg/deepregex/6.sl
infeasible
delis ./benchmark/regel/cfg/deepregex/6.sl  4.20s user 0.04s system 105% cpu 4.025 total
(base) [main][~/Codes/Rust/delis]$ time delis ./benchmark/regel/cfg/deepregex/6.sl --dump-chc | z3 -in
sat
delis ./benchmark/regel/cfg/deepregex/6.sl --dump-chc  0.16s user 0.01s system 57% cpu 0.304 total
z3 -in  0.48s user 0.06s system 76% cpu 0.717 total
(base) [main][~/Codes/Rust/delis]$ time delis ./benchmark/regel/cfg/deepregex/6.sl --dump-sygus | cvc5 --lang=sygus2
^Ccvc5 interrupted by user.

delis ./benchmark/regel/cfg/deepregex/6.sl --dump-sygus  0.13s user 0.01s system 98% cpu 0.148 total
```

```sh
# with dryadsynth-bv
$ time delis ./benchmark/alpharegex/re/no22.sl
(
(define-fun regex () RegLan (re.++ (re.* re.allchar) (re.union (re.++ char.0 char.0) (re.++ re.allchar char.1))))
)
delis ./benchmark/alpharegex/re/no22.sl  121.63s user 0.55s system 197% cpu 1:01.84 total

$ time delis ./benchmark/alpharegex/re/no22.sl --dump-sygus | cvc5 --lang=sygus2   
(
(define-fun r () (_ BitVec 441) (re.++ (re.* re.allchar) (re.union (re.++ char.0 char.0) (re.++ re.allchar char.1))))
)
delis ./benchmark/alpharegex/re/no22.sl --dump-sygus  0.05s user 0.01s system 83% cpu 0.073 total
cvc5 --lang=sygus2  44.93s user 0.44s system 99% cpu 45.436 total
```

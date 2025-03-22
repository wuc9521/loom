(declare-const x Int)
(assert (= x 100000000000000000000000000000000000000000000000000000000)) ; How does solver even handle this? What does Int mean in solver?
(check-sat)
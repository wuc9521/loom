(
(define-fun regex () RegLan (let ((_let_1 (re.union (re.union (re.range "0" "9") (re.range "a" "z")) (re.range "A" "Z")))) (re.++ _let_1 (re.++ (re.+ re.allchar) _let_1))))
)

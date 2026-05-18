(
(define-fun regex () RegLan (re.++ (re.inter (re.range "a" "z") (re.union (re.union (re.range "0" "9") (re.range "a" "f")) (re.range "A" "F"))) (re.+ re.allchar)))
)

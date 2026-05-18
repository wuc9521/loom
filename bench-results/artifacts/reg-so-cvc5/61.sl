(
(define-fun regex () RegLan (re.+ (re.++ (re.union (re.union (re.range "0" "9") (re.range "a" "z")) (re.range "A" "Z")) (re.opt re.allchar))))
)

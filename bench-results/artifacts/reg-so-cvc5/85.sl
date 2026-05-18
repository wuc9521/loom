(
(define-fun regex () RegLan (re.+ (re.union (re.union (re.range "a" "z") (re.range "A" "Z")) (re.++ (re.range "0" "9") re.allchar))))
)

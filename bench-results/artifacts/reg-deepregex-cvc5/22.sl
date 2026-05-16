(
(define-fun regex () RegLan (re.+ (re.union (re.range "0" "9") (re.union (re.range "a" "z") (re.++ re.allchar re.allchar)))))
)

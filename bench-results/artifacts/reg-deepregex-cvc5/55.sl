(
(define-fun regex () RegLan (re.+ (re.union (str.to_re "!") (re.union (str.to_re "@") (re.++ (re.union (re.range "a" "z") (re.range "A" "Z")) re.allchar)))))
)

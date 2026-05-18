(
(define-fun regex () RegLan (re.++ (re.+ (re.union (re.range "a" "z") (str.to_re "!"))) (str.to_re "@")))
)

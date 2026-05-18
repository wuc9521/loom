(
(define-fun regex () RegLan (re.+ (re.union (str.to_re "!") (re.++ (re.union (re.union (re.range "0" "9") (re.range "a" "f")) (re.range "A" "F")) (re.* (re.range "a" "z"))))))
)

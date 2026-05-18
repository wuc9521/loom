(
(define-fun regex () RegLan (re.++ (re.union (re.union (re.range "a" "z") (re.range "A" "Z")) (str.to_re "!")) re.allchar))
)

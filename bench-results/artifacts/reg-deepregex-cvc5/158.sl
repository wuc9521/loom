(
(define-fun regex () RegLan (re.++ re.allchar (re.union (re.union (re.range "a" "z") (re.range "A" "Z")) (re.++ (str.to_re "@") (re.opt re.allchar)))))
)

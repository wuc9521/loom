(
(define-fun regex () RegLan (re.++ re.allchar (re.++ (re.union (re.range "a" "z") (str.to_re "!")) (re.opt re.allchar))))
)

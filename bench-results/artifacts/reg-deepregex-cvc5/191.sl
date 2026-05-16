(
(define-fun regex () RegLan (re.++ re.allchar (re.union (str.to_re "!") (re.++ (re.range "a" "z") (re.* re.allchar)))))
)

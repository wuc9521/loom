(
(define-fun regex () RegLan (re.++ (re.union (re.range "A" "Z") (re.union (str.to_re "!") (str.to_re "@"))) re.allchar))
)

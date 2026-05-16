(
(define-fun regex () RegLan (re.union (str.to_re "!") (re.++ (re.+ re.allchar) (re.range "0" "9"))))
)

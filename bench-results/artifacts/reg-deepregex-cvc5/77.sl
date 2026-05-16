(
(define-fun regex () RegLan (re.++ (re.* re.allchar) (re.union (str.to_re "!") (str.to_re "@"))))
)

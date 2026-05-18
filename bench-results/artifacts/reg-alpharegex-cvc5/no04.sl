(
(define-fun regex () RegLan (re.++ (str.to_re "0") (re.++ (re.* re.allchar) (str.to_re "1"))))
)

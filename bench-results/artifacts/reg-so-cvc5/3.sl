(
(define-fun regex () RegLan (re.+ (re.++ re.allchar (re.union (str.to_re ".") (re.++ re.allchar re.allchar)))))
)

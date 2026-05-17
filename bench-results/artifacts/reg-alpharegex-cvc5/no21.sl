(
(define-fun regex () RegLan (re.* (re.++ re.allchar (re.union (str.to_re "0") (re.++ re.allchar (re.++ re.allchar re.allchar))))))
)

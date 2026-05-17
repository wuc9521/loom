(
(define-fun regex () RegLan (re.++ (re.union (str.to_re "0") (re.++ (str.to_re "1") re.allchar)) (re.* (re.++ re.allchar re.allchar))))
)

(
(define-fun regex () RegLan (re.* (re.++ (re.union (str.to_re "1") (re.++ re.allchar re.allchar)) re.allchar)))
)

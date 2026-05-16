(
(define-fun regex () RegLan (re.+ (re.++ (re.union (str.to_re " ") (re.++ re.allchar re.allchar)) re.allchar)))
)

(
(define-fun regex () RegLan (re.++ re.allchar (re.+ (re.union (str.to_re " ") (re.++ (re.range "0" "9") re.allchar)))))
)

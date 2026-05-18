(
(define-fun regex () RegLan (re.+ (re.++ re.allchar (re.++ (re.union (re.range "0" "9") (str.to_re "-")) re.allchar))))
)

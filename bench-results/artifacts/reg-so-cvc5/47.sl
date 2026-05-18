(
(define-fun regex () RegLan (re.++ re.allchar (re.++ (re.+ (re.union (re.range "0" "9") (str.to_re "."))) re.allchar)))
)

(
(define-fun regex () RegLan (re.+ (re.union (re.range "A" "Z") (re.union (re.union (re.union (re.range "0" "9") (re.range "a" "f")) (re.range "A" "F")) (re.++ (str.to_re "!") re.allchar)))))
)

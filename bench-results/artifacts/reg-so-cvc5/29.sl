(
(define-fun regex () RegLan (re.++ re.allchar (re.* (re.union (re.range "0" "9") (re.++ (str.to_re ",") re.allchar)))))
)

(
(define-fun regex () RegLan (re.+ (re.++ re.allchar (re.union re.allchar (re.++ (str.to_re "_") re.allchar)))))
)

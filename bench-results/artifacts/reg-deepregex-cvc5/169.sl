(
(define-fun regex () RegLan (let ((_let_1 (re.range "a" "z"))) (re.++ re.allchar (re.union _let_1 (re.++ (re.+ re.allchar) (re.union _let_1 (re.range "A" "Z")))))))
)

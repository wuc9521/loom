(
(define-fun regex () RegLan (let ((_let_1 (re.range "a" "z"))) (re.++ re.allchar (re.++ (re.union _let_1 (re.range "A" "Z")) (re.++ _let_1 re.allchar)))))
)

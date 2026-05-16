(
(define-fun regex () RegLan (let ((_let_1 (re.range "a" "z"))) (re.+ (re.union (re.union (re.union (re.range "0" "9") _let_1) (re.range "A" "Z")) (re.++ re.allchar (re.++ _let_1 re.allchar))))))
)

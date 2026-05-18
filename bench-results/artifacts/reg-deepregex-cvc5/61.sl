(
(define-fun regex () RegLan (let ((_let_1 (re.range "A" "Z"))) (re.+ (re.++ re.allchar (re.++ (re.union (re.union (re.range "0" "9") (re.range "a" "z")) _let_1) (re.opt _let_1))))))
)

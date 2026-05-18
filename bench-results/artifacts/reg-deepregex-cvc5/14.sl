(
(define-fun regex () RegLan (let ((_let_1 (re.range "A" "Z"))) (re.++ re.allchar (re.++ (re.+ (re.union (re.range "0" "9") _let_1)) _let_1))))
)

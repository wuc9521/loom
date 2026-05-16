(
(define-fun regex () RegLan (let ((_let_1 (re.range "A" "Z"))) (re.+ (re.union _let_1 (re.++ (re.union (re.range "a" "z") _let_1) re.allchar)))))
)

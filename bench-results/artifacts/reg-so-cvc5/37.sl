(
(define-fun regex () RegLan (let ((_let_1 (re.range "A" "Z"))) (re.+ (re.union _let_1 (re.++ re.allchar (re.union (re.union (re.range "a" "z") _let_1) (str.to_re "@")))))))
)

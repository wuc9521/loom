(
(define-fun regex () RegLan (let ((_let_1 (re.range "a" "z"))) (re.+ (re.union _let_1 (re.++ (re.union (re.union (re.union (re.range "0" "9") _let_1) (re.range "A" "Z")) (str.to_re "_")) re.allchar)))))
)

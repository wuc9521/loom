(
(define-fun regex () RegLan (let ((_let_1 (re.range "0" "9"))) (re.+ (re.union (re.union (re.union _let_1 (re.range "a" "f")) (re.range "A" "F")) (re.++ (str.to_re ":") (re.++ (re.union (re.union _let_1 (re.range "a" "z")) (re.range "A" "Z")) re.allchar))))))
)

(
(define-fun regex () RegLan (let ((_let_1 (re.range "0" "9"))) (re.+ (re.union _let_1 (re.++ (re.union (re.union (re.union _let_1 (re.range "a" "z")) (re.range "A" "Z")) (str.to_re "!")) re.allchar)))))
)

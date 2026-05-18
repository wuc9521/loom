(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "0"))) (re.* (re.union _let_1 (re.++ (str.to_re "1") _let_1)))))
)

(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "1"))) (let ((_let_2 (str.to_re "0"))) (re.* (re.union _let_2 (re.++ _let_1 (re.++ (re.* _let_2) _let_1)))))))
)

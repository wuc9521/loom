(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "1"))) (let ((_let_2 (str.to_re "0"))) (re.++ (re.opt _let_2) (re.++ (re.* (re.++ _let_1 _let_2)) (re.opt _let_1))))))
)

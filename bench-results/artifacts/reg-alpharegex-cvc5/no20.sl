(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "0"))) (re.++ (re.* (re.++ _let_1 re.allchar)) (re.opt _let_1))))
)

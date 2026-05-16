(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "%"))) (re.++ _let_1 (re.++ (re.* re.allchar) _let_1))))
)

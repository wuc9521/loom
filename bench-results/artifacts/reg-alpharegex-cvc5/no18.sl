(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "1"))) (re.++ re.allchar (re.* (re.++ _let_1 (re.opt (re.++ _let_1 (str.to_re "0"))))))))
)

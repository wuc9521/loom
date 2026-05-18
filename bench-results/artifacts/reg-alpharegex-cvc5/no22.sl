(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "0"))) (re.++ (re.* re.allchar) (re.union (re.++ _let_1 _let_1) (re.++ re.allchar (str.to_re "1"))))))
)

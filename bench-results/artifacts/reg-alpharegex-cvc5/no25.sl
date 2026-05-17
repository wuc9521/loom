(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "0"))) (re.union _let_1 (re.* (re.++ (re.union (str.to_re "1") (re.++ _let_1 _let_1)) re.allchar)))))
)

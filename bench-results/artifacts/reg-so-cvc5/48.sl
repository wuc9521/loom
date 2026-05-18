(
(define-fun regex () RegLan (let ((_let_1 (re.range "0" "9"))) (re.+ (re.++ _let_1 (re.* (re.++ re.allchar _let_1))))))
)

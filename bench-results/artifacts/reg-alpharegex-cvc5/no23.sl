(
(define-fun regex () RegLan (let ((_let_1 (str.to_re "0"))) (re.++ (re.* (re.++ (re.union _let_1 (re.++ re.allchar re.allchar)) re.allchar)) _let_1)))
)

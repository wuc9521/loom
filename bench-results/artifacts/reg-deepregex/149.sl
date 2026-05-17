(
(define-fun regex () RegLan (re.plus (re.union char.digit (re.++ (re.union char.alnum char.x21) char.any))))
)

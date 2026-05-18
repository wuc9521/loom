(
(define-fun regex () RegLan (re.++ (re.plus char.upper) (re.++ (re.union char.digit (re.union char.x21 char.x40)) (re.* char.any))))
)

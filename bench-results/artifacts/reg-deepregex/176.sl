(
(define-fun regex () RegLan (re.++ (re.union char.digit (re.union char.x21 (re.++ char.any char.lower))) (re.* char.any)))
)

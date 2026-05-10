(
(define-fun regex () RegLan (re.union (re.plus char.hex) (re.++ char.x21 (re.plus char.any))))
)

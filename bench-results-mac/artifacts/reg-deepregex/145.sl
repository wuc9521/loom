(
(define-fun regex () RegLan (re.++ (re.union char.upper (re.inter char.lower char.hex)) (re.plus char.any)))
)

(
(define-fun regex () RegLan (re.plus (re.union char.upper (re.++ char.any (re.union char.x21 (re.plus char.hex))))))
)

(
(define-fun regex () RegLan (re.plus (re.union char.x21 (re.++ char.hex (re.* char.lower)))))
)

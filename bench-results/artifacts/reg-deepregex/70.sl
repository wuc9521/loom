(
(define-fun regex () RegLan (re.plus (re.++ char.any (re.++ char.alpha (re.union char.upper (re.* char.digit))))))
)

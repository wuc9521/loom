(
(define-fun regex () RegLan (re.++ (re.* char.alnum) (re.union char.upper (re.++ char.x5E (re.plus char.any)))))
)

(
(define-fun regex () RegLan (re.++ (re.union char.alnum (re.union char.x23 char.x40)) (re.* char.any)))
)

(
(define-fun regex () RegLan (re.++ (re.* char.any) (re.++ (re.union char.digit char.lower) char.x21)))
)

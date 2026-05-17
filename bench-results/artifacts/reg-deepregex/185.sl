(
(define-fun regex () RegLan (re.++ (re.opt char.any) (re.++ (re.union char.digit (re.union char.x21 char.x40)) (re.opt char.any))))
)

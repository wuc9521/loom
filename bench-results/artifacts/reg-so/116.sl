(
(define-fun regex () RegLan (re.plus (re.++ (re.opt char.any) (re.union char.lower (re.++ char.x25 (re.plus char.any))))))
)

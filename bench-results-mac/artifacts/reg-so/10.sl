(
(define-fun regex () RegLan (re.* (re.inter (re.union char.upper (re.++ char.any (re.plus char.any))) (re.union (re.plus char.alnum) (re.++ char.any char.any)))))
)

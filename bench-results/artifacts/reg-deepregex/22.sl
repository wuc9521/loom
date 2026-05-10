(
(define-fun regex () RegLan (re.inter (re.plus char.any) (re.union (re.++ char.any (re.plus char.any)) (re.++ (re.opt char.digit) (re.opt char.lower)))))
)

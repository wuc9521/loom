(
(define-fun regex () RegLan (re.inter (re.union (re.++ char.any (re.plus char.any)) (re.++ (re.opt char.lower) (re.opt char.x21))) (re.union (re.++ (re.opt char.alnum) char.any) (re.++ (re.plus char.any) char.alnum))))
)

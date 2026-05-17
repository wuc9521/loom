(
(define-fun regex () RegLan (re.plus (re.++ (re.opt char.digit) (re.union char.alpha (re.++ char.any char.any)))))
)

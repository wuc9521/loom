(
(define-fun regex () RegLan (re.plus (re.++ char.digit (re.++ (re.opt (re.++ char.digit char.digit)) (re.++ (re.opt char.x2E) (re.++ char.digit char.digit))))))
)

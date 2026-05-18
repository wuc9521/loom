(
(define-fun regex () RegLan (re.plus (re.++ char.lower (re.opt (re.++ (re.plus char.any) char.upper)))))
)

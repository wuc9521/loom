(
(define-fun regex () RegLan (re.plus (re.++ (re.inter char.upper char.hex) (re.++ char.any (re.opt char.any)))))
)

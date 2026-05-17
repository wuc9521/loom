(
(define-fun regex () RegLan (re.++ (re.* char.digit) (re.++ (re.opt (re.++ char.x2C char.any)) (re.++ (re.opt char.any) char.digit))))
)

(
(define-fun regex () RegLan (re.++ char.any (re.opt (re.++ (re.union char.space char.x21) (re.* char.any)))))
)

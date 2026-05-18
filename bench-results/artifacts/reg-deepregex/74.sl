(
(define-fun regex () RegLan (re.opt (re.++ (re.union char.alpha (re.union char.space char.x40)) (re.plus char.any))))
)

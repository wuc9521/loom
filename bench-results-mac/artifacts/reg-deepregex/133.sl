(
(define-fun regex () RegLan (re.plus (re.union (re.++ char.any char.digit) (re.++ (re.plus char.alpha) char.lower))))
)

(
(define-fun regex () RegLan (re.union (re.++ char.hex (re.plus char.any)) (re.++ (re.plus char.any) char.digit)))
)

(
(define-fun regex () RegLan (re.plus (re.union char.alnum (re.++ char.x21 (re.++ (re.* char.any) char.x40)))))
)

(
(define-fun regex () RegLan (re.union (re.plus char.alpha) (re.++ (re.plus char.any) char.upper)))
)
